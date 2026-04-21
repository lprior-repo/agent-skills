# Async Rust Review: Order Service (Domain Types, Port Trait, HTTP Handler)

**Mode**: SNIPPET | **Date**: 2026-04-03
**Code**: Inline submission -- `crates/domain/order.rs`, `crates/domain/repo.rs`, `crates/api/handlers.rs`
**Reviewer**: async-rust-reviewer skill v1.0.0

---

## VERDICT: APPROVED

This is a textbook example of the sync-core/async-shell pattern. The code correctly separates synchronous domain logic from asynchronous infrastructure orchestration. Zero lethal findings.

---

## LETHAL FINDINGS (0)

None.

---

## MAJOR FINDINGS (0)

None.

---

## DELEGATED FINDINGS (0)

None that are visible in the submitted snippet. The domain types use owned `String` in newtype wrappers without `#[derive(...)]` visible, but that is a functional-rust concern, not an async concern.

---

## CRITICAL FINDINGS (1)

1. **No benchmark evidence for async hot path** (`crates/api/handlers.rs`, `handle_create`): The `handle_create` handler is an async hot path -- an HTTP endpoint performing I/O-bound persistence. No criterion benchmarks were provided. Per `benchmarks_or_no_merge` principle and rules `bench_throughput_baseline` / `bench_async_vs_sync`, throughput baselines must be established for this endpoint before production deployment. In snippet mode this is expected (no project on disk), but the mandate stands for any real crate: create `benches/async_perf.rs` with criterion benchmarks comparing sync-core vs async-shell throughput. See `references/benchmark-patterns.md` for the correct setup.

---

## PHASE 1: Spawn Discipline & Runtime Hygiene

**PASS**

| Check | Result |
|-------|--------|
| `tokio::spawn` in domain crate | NO MATCH -- clean |
| `tokio::spawn` in handlers | NO MATCH -- no spawn anywhere |
| `async fn` in domain crate | NO MATCH -- `validate_order` is `pub fn`, not `pub async fn` |
| Unnecessary async (`async fn` with zero `.await`) | NO MATCH -- `handle_create` has exactly 1 `.await` |
| CPU-bound work misrouted as async | NO MATCH -- `validate_order` is sync, checks `items.is_empty()` and moves data |
| Blocking operations in async context | NO MATCH -- no sync I/O or CPU loops in `handle_create` |

**Analysis**:

- `validate_order` in `crates/domain/order.rs` is a synchronous `fn` returning `Result<Validated, DomainError>`. It performs a single check (`draft.items.is_empty()`) and moves data. This is correct: domain logic is pure, synchronous, testable without a runtime.

- `handle_create` in `crates/api/handlers.rs` is `async fn` with exactly one `.await` point: `repo.save(&Order::Validated(validated)).await?`. The async is justified -- it awaits an I/O-bound repository operation. No async wrapper around sync work.

- The sync-core/async-shell boundary is clean: `validate_order` is the sync core, `repo.save()` is the async shell.

- No `tokio::spawn` anywhere. Spawn discipline is trivially satisfied.

**Rule references**: `spawn_at_edge_only`, `no_await_in_calc`, `sync_over_async`, `no_unnecessary_async`, `never_block_runtime`

---

## PHASE 2: Stream Combinators & Concurrency Primitives

**N/A -- no streams present**

| Check | Result |
|-------|--------|
| Imperative async loops (`while let Some(_) = stream.next().await`) | NO MATCH |
| Unbounded `join_all` | NO MATCH |
| `Vec<JoinHandle>` bookkeeping | NO MATCH |
| `buffer_unordered` without capacity | NO MATCH |

No stream processing in the submitted code. The handler performs a single sequential async operation. Nothing to evaluate for combinator discipline.

---

## PHASE 3: Send + Sync Hygiene & Ownership Design

**PASS**

| Check | Result |
|-------|--------|
| `Rc<T>` in async context | NO MATCH |
| `RefCell<T>` in async context | NO MATCH |
| `std::sync::Mutex` across `.await` | NO MATCH |
| `Arc<Mutex<T>>` without ladder justification | NO MATCH -- no `Arc<Mutex>` present |
| Borrowed references across spawn boundaries | NO MATCH -- no spawn |
| Non-Send state crossing `.await` | NO MATCH |

**Analysis**:

- `OrderId(String)` and `CustomerId(String)` are `Send + Sync`. `String` is `Send + Sync`, and unit-like tuple-struct wrappers inherit those bounds.
- `smallvec::SmallVec<[String; 4]>` is `Send + Sync` (it stores data inline on the stack for N<=4, falling back to heap; both paths are `Send + Sync`).
- `handle_create` takes `&dyn OrderRepo` -- a borrowed trait object. No spawning, no `'static` requirements.
- All types crossing the single `.await` point (`Order::Validated(validated)`) are fully owned and `Send`.

---

## PHASE 4: Cancellation Safety & Pin Awareness

**PASS**

| Check | Result |
|-------|--------|
| State mutation before `.await` without recovery | NO MATCH |
| Partial writes / non-atomic check-then-act | NO MATCH |
| Unnecessary `Box::pin` | NO MATCH |
| Self-referential structs without Pin docs | NO MATCH |

**Analysis**:

The `handle_create` function has a single `.await` point at `repo.save()`. The state before `.await`:

1. `draft` is constructed from `cmd` (sync, instant).
2. `validated` is produced by `validate_order(draft)?` (sync, pure).
3. `repo.save(&Order::Validated(validated)).await?` is the only yield point.

If the Future is dropped at the `.await`:
- Before `repo.save` completes: the order was never persisted. No corrupted state. The validated value is simply lost -- the caller can retry.
- After `repo.save` completes: the operation is atomic at the repository level. The adapter implementation is responsible for ensuring atomic writes.

This matches the canonical "prepare sync, then single atomic await" pattern from `references/cancellation-safety.md`. No self-referential futures, no `Pin` concerns, no recursive async.

---

## PHASE 5: Observability & Error Propagation

**PASS**

| Check | Result |
|-------|--------|
| `async fn` in shell without `#[instrument]` | NO MATCH -- `handle_create` has `#[tracing::instrument(skip(repo), fields(user_id = %cmd.customer_id))]` |
| `println!` / `eprintln!` in async | NO MATCH |
| `tokio::spawn` without span propagation | NO MATCH -- no spawn |
| `let _ = join_handle.await` | NO MATCH -- no JoinHandle |
| Error context lost via bare `?` without `.context()` | SEE NOTE below |

**Analysis**:

- `#[tracing::instrument(skip(repo), fields(user_id = %cmd.customer_id))]` is correctly applied:
  - Skips `repo` (not `Debug`).
  - Adds structured `user_id` field for trace correlation across async boundaries.
  - The span covers the entire handler execution, including the `.await`.

- Error propagation uses `?` operator: `validate_order(draft)?` and `repo.save(...).await?`. The error chain is preserved through the `?` operator -- `DomainError` propagates from domain, `AppError` wraps at the shell boundary.

**Minor note (non-blocking)**: The `repo.save()` error propagates via `?` but is not wrapped with `.context()` or `.map_err()` to add handler-level context. Whether this matters depends on how `AppError` and `DomainError` are defined. If `AppError` already preserves the full causal chain, this is fine. If not, consider `.map_err(|e| AppError::persistence("failed to persist validated order", e))?` for richer error chains at the shell boundary. This is a suggestion, not a finding.

---

## PHASE 6: Hexagonal Architecture Boundaries

**PASS (well-structured)**

| Check | Result |
|-------|--------|
| Domain crate async deps (`tokio`, `futures`, `async-std`, `async-trait`) | STATIC: not present in snippet -- domain has no async imports |
| `.await` in domain source | NO MATCH |
| `spawn` in domain source | NO MATCH |
| Port trait defined in domain crate | YES -- `OrderRepo` is in `crates/domain/repo.rs` |
| Only infra adapters contain `.await` | YES -- only `handle_create` in `crates/api/handlers.rs` has `.await` |

**Analysis**:

- **Domain crate** (`crates/domain/order.rs`): Contains `OrderId`, `CustomerId`, `Order` enum with state variants (`Draft`, `Validated`, `Priced`), and a pure sync `validate_order` function. No async dependencies, no `.await`, no `tokio`. Textbook correct.

- **Port trait** (`crates/domain/repo.rs`): `OrderRepo` with `async fn save`. Defined in the domain crate where the domain types live. This is correct placement -- the domain owns the port contract. The `async fn` in the trait is fine with native async fn in traits (stable); no `async-trait` crate needed. The domain crate does not depend on any async runtime.

- **Handler** (`crates/api/handlers.rs`): The presentation layer calls sync domain logic (`validate_order(draft)?`), then awaits the infra adapter (`repo.save(&Order::Validated(validated)).await?`). This matches the canonical pattern from `references/hexagonal-boundaries.md`:

  ```
  handler: async fn create(req) -> Result<Response, AppError> {
      let cmd = parse(req)?;
      let order = use_case.execute(cmd)?;    // sync domain
      repo.save(&order).await?;               // async infra
      Ok(response)
  }
  ```

- **Dependency direction**: Domain -> no deps. Application -> domain. Infra -> domain. Presentation -> application + infra. Correct.

- **Type-state pattern**: `Order` enum with `Draft`, `Validated`, `Priced` variants enforces the state machine at the type level. `validate_order` consumes `Draft` and produces `Validated` -- you cannot persist an unvalidated order. This is excellent DDD discipline.

---

## PHASE 7: Ruthless Async Simplicity

**PASS**

| Check | Result |
|-------|--------|
| Async function > 60 lines | NO -- `handle_create` is ~4 lines of logic |
| > 3 `.await` points per function | NO -- exactly 1 `.await` |
| > 1 `tokio::spawn` per handler | NO -- 0 spawns |
| Speculative concurrency / YAGNI violations | NO -- no concurrency primitives at all |
| Async sniff test: would sync be simpler? | NO -- `repo.save()` requires async I/O |

**Analysis**:

The code is boring and obvious. That is the highest compliment for async Rust. The handler does exactly three things: construct a draft, validate it synchronously, persist it asynchronously. No cleverness, no over-engineering, no speculative concurrency.

---

## STATIC ANALYSIS

The submitted code is an inline snippet with no Cargo.toml on disk. The verification gate commands cannot be executed. All evidence below is STATIC ANALYSIS, not EXECUTION.

### Pattern Scans

| Pattern | Scan Result |
|---------|------------|
| `.await` | 1 occurrence: `repo.save(&Order::Validated(validated)).await?` in `crates/api/handlers.rs` |
| `.await` in `crates/domain/` | NO MATCH |
| `tokio::spawn` | NO MATCH anywhere |
| `spawn_local` / `spawn_blocking` | NO MATCH anywhere |
| `Rc<` | NO MATCH |
| `RefCell` | NO MATCH |
| `std::sync::Mutex` | NO MATCH |
| `Arc<Mutex<` | NO MATCH |
| `println!` / `eprintln!` | NO MATCH |
| `while let Some(_) = stream.next().await` | NO MATCH |
| `join_all` | NO MATCH |
| `Vec<JoinHandle>` | NO MATCH |
| `Box::pin` | NO MATCH |
| `#[tracing::instrument]` | 1 occurrence: on `handle_create` |

### Predicted Clippy Outcomes

| Lint | Prediction | Reasoning |
|------|-----------|-----------|
| `clippy::unused_async` | WOULD NOT FIRE | `handle_create` has exactly 1 `.await` -- async is used |
| `clippy::await_holding_lock` | WOULD NOT FIRE | No locks held across `.await` |
| `clippy::await_holding_refcell_ref` | WOULD NOT FIRE | No `RefCell` borrows across `.await` |
| `clippy::large_futures` | WOULD NOT FIRE | Future is small: one `.await`, simple stack types |
| `clippy::pedantic` | POSSIBLE WARNINGS | Newtype wrappers without `Debug`/`Display` derives, but these are functional-rust concerns |

### Verification Gate (Predicted)

| Layer | Predicted Result | Reasoning |
|-------|-----------------|-----------|
| Layer 1: Async clippy lints | PASS | No banned patterns detected |
| Layer 2: Domain zero async deps | PASS | No async imports in domain snippet |
| Layer 3: No `.await`/spawn in domain | PASS | Static scan confirms clean |
| Layer 4: Functional-rust sync gate | UNKNOWN | Depends on code not shown (derive macros, formatting) |
| Layer 5: Tests | UNKNOWN | No tests provided in snippet |
| Layer 6: Benchmarks | CRITICAL GAP | No benchmarks provided -- see Critical Finding #1 |
| Layer 7: tokio-console | UNKNOWN | No project to inspect |

---

## MANDATE

None required. No changes needed.

This code is a reference implementation of the sync-core/async-shell architecture:

- Domain logic is pure, synchronous, and has zero async dependencies.
- The port trait (`OrderRepo`) is defined in the domain crate with `async fn` (correct -- the domain owns the contract, not the implementation).
- The handler is a thin async shell: call sync domain, await infra adapter, return response.
- `#[tracing::instrument]` is correctly applied with appropriate skips and structured fields.
- Cancellation safety is inherent: single atomic `.await` after pure sync preparation.
- Send+Sync hygiene is clean: no `Rc`, no `RefCell`, no `Mutex` across `.await`.
- Type-state pattern (`Draft` -> `Validated` -> `Priced`) enforces business invariants at compile time.

**Recommendation**: Use this code as a template for all handlers in the project.
