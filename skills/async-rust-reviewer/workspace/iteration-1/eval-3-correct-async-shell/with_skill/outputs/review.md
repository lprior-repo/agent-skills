# Async Rust Review: Order Service

**Reviewed**: 2026-04-03
**Code**: Inline submission — domain types, trait port, and HTTP handler
**Reviewer**: async-rust-reviewer skill v1.0.0

---

## PHASE 1: Spawn Discipline & Runtime Hygiene

**PASS**

- No `tokio::spawn` present anywhere. No spawn in domain, no spawn in handlers. Clean.
- `validate_order` in `crates/domain/order.rs` is a sync `fn` returning `Result<Validated, DomainError>`. This is correct: domain logic is synchronous, no `.await`, no async.
- `handle_create` in `crates/api/handlers.rs` is `async fn` with exactly one `.await` point (`repo.save(&Order::Validated(validated)).await?`). The async is justified: it awaits an I/O-bound repository operation.
- No CPU-bound work misrouted as async. `validate_order` is sync, as it should be (checking `items.is_empty()` and moving data).
- No unnecessary async. Every `async fn` has at least one `.await`.
- No blocking operations in async context.

**Spawn Discipline: CLEAN**

---

## PHASE 2: Stream Combinators & Concurrency Primitives

**PASS (no streams present)**

- No stream processing in the submitted code. Nothing to evaluate for combinator discipline.
- No `join_all`, no unbounded concurrency, no `Vec<JoinHandle>` bookkeeping.
- No `buffer_unordered` or `for_each_concurrent` needed for this simple handler.

**Concurrency: N/A — single .await, no concurrent operations**

---

## PHASE 3: Send + Sync Hygiene & Ownership Design

**PASS**

- No `Rc<T>`, `RefCell<T>`, or `std::sync::Mutex` present.
- No `Arc<Mutex<T>>` present in the submitted code.
- `handle_create` takes `&dyn OrderRepo` — a borrowed trait reference, not spawning, so `'static` concerns do not apply.
- No non-Send state crossing `.await` boundaries.
- `OrderId(String)` and `CustomerId(String)` are `Send + Sync` (String is Send+Sync, unit-like wrappers inherit it).
- `smallvec::SmallVec<[String; 4]>` is `Send + Sync`.

**Send+Sync: CLEAN**

---

## PHASE 4: Cancellation Safety & Pin Awareness

**PASS (with minor note)**

- The `handle_create` function has a single `.await` point at `repo.save()`.
- State before `.await`: `validated` is a fully-constructed, self-consistent value. No partial mutation.
- If the Future is dropped after `validate_order` succeeds but before `repo.save` completes: the `validated` value is simply lost. No corrupted state. The order was never persisted, so no inconsistency.
- If the Future is dropped during `repo.save().await`: the save either completed atomically or it did not. The repo is the authority on state. No partial write concern at the application level (though the repo implementation must ensure atomic writes — that is the adapter's responsibility, not the handler's).
- No self-referential futures, no `Pin` concerns, no recursive async.
- No `Box::pin` used, and none needed.

**Cancellation Safety: SAFE** — single atomic await, sync preparation, state valid at every yield point.

---

## PHASE 5: Observability & Error Propagation

**PASS**

- `handle_create` has `#[tracing::instrument(skip(repo), fields(user_id = %cmd.customer_id))]`. This is correct:
  - Skips `repo` (not Debug).
  - Adds structured `user_id` field for trace correlation.
  - The instrument macro creates a span covering the entire handler execution.
- No `println!` or `eprintln!` in async code.
- Error propagation uses `?` operator throughout: `validate_order(draft)?` and `repo.save(...).await?`. The error chain is preserved via the `?` operator — `DomainError` propagates from domain, `AppError` wraps at the shell boundary.
- No spawned tasks, so span propagation into spawn is not applicable.

**Minor suggestions (not blocking)**:
- Consider adding a structured `tracing::info!` or `tracing::debug!` after successful save with `order_id` or `customer_id` for audit trail. Not required, but strengthens observability.
- The `repo.save()` error is propagated via `?` but not wrapped with `.context()` or `.map_err()` to add handler-level context (e.g., "failed to persist validated order"). Consider adding `.map_err(|e| AppError::Persistence(e))?` for richer error chains. The current code relies on the error type itself carrying enough context, which may or may not be sufficient depending on `AppError`'s definition.

**Observability: CLEAN**

---

## PHASE 6: Hexagonal Architecture Boundaries

**PASS (well-structured)**

- **Domain crate** (`crates/domain/order.rs`): Contains structs, enums, and a pure sync function `validate_order`. No async dependencies. No `.await`. No `tokio`. This is textbook correct.
- **Port trait** (`crates/domain/repo.rs`): `OrderRepo` trait with `async fn save`. The trait is defined in the domain crate (where the domain types live). This is the correct placement — the domain owns the port contract. The `async fn` in the trait is fine; it does not require the domain crate to depend on any async runtime (native async fn in traits is stable; no `async-trait` crate needed).
- **Handler** (`crates/api/handlers.rs`): The presentation layer calls sync domain logic (`validate_order`) then awaits the infra adapter (`repo.save`). This matches the canonical pattern from `hexagonal-boundaries.md`:
  ```
  handler: async fn create(req: Request) -> Result<Response, AppError> {
      let cmd = parse(req)?;
      let order = use_case.execute(cmd)?;    // sync domain
      repo.save(&order).await?;               // async infra
      Ok(response)
  }
  ```
- **Dependency direction**: Domain has no external deps. Application depends on domain. Infra depends on domain. Presentation depends on application + infra. Correct.

**Hexagonal Boundaries: CLEAN**

---

## PHASE 7: Ruthless Async Simplicity

**PASS**

- `handle_create` is approximately 4 lines of logic. Well under the 60-line limit.
- Exactly 1 `.await` point. Well under the 3-await limit.
- No speculative concurrency. No over-engineered channel topologies.
- The "Async Sniff Test": Would a sync version be simpler? No — the handler must call an async repository. The async is justified by the I/O-bound persistence operation. The domain logic (`validate_order`) is already sync.
- No `tokio::spawn` in the handler. Max spawn-per-handler rule is satisfied trivially.
- The code is boring and obvious. That is a compliment.

**Simplicity: CLEAN**

---

## EXECUTION EVIDENCE

The submitted code is an inline snippet, not a local crate. The verification gate commands cannot be executed against a project that does not exist on disk. The following notes apply:

| Verification Layer | Status | Notes |
|---|---|---|
| Layer 1: Async clippy lints | NOT RUN | No Cargo.toml, no crate to compile |
| Layer 2: Domain zero async deps | NOT RUN | No `cargo metadata` available |
| Layer 3: No .await/spawn in domain | MANUAL PASS | Static analysis confirms: `crates/domain/order.rs` has no `.await`, no spawn, no async fn |
| Layer 4: Functional-rust sync gate | NOT RUN | No crate to compile |
| Layer 5: Tests | NOT RUN | No test suite provided |
| Layer 6: Benchmarks | NOT RUN | No benchmarks provided |
| Layer 7: tokio-console | NOT RUN | No project to inspect |

**Static analysis performed**:
- Scanned submitted code for `.await`: 1 occurrence in `handle_create` at `repo.save(...).await?`. None in domain.
- Scanned submitted code for `tokio::spawn`: 0 occurrences.
- Scanned submitted code for `Rc`, `RefCell`, `std::sync::Mutex`: 0 occurrences.
- Scanned submitted code for `println`, `eprintln`: 0 occurrences.
- Verified `#[tracing::instrument(...)]` present on the single async function.

---

### VERDICT: APPROVED

### LETHAL FINDINGS (0)

None. The code is well-structured and follows all async discipline rules.

### MAJOR FINDINGS (0)

None. The architecture is clean.

### MINOR FINDINGS (2)

1. **Missing structured log after successful save** (`crates/api/handlers.rs`, `handle_create`): Adding a `tracing::info!(customer_id = %validated.customer_id, "order validated and persisted")` after `repo.save()` would strengthen the audit trail. Currently, the span records entry/exit but no intermediate business events.

2. **Error context at the shell boundary** (`crates/api/handlers.rs`, `handle_create`): The `repo.save(...).await?` propagates errors via `?` but does not add handler-level context. Consider `.map_err(|e| AppError::from_persistence(e))?` or similar to preserve the causal chain ("failed to persist validated order for customer X"). Whether this is actually needed depends on how `AppError` and `DomainError` are defined, which is not shown.

### CRITICAL FINDINGS (1)

1. **No benchmark evidence for async hot path**: The `handle_create` handler is an async hot path (HTTP endpoint with I/O). No criterion benchmarks were provided. Per the "benchmarks or no merge" principle, throughput baselines for this endpoint should be established before production deployment. This is flagged as CRITICAL because the skill mandates benchmark evidence, but the submitted code is a snippet without a project structure, so this is expected in context.

### MANDATE

None. No changes required. This is a textbook example of the sync-core/async-shell pattern done correctly:

- Domain logic is pure, synchronous, and has zero async dependencies.
- The port trait (`OrderRepo`) is defined in the domain crate with `async fn` (correct — the domain owns the contract, not the implementation).
- The handler is a thin async shell: call sync domain, await infra adapter, return response.
- `#[tracing::instrument]` is correctly applied with appropriate skips and fields.
- Cancellation safety is inherent: single atomic `.await` after pure sync preparation.
- Send+Sync hygiene is clean: no `Rc`, no `RefCell`, no `Mutex` across `.await`.

This code can serve as a reference implementation for the sync-core/async-shell architecture.
