# Async Rust Review: crates/domain/order_processor.rs
**Mode**: SNIPPET | **Date**: 2026-04-03

## VERDICT: REJECTED

This code fails on nearly every axis of async Rust discipline. The domain crate has no business containing `tokio::spawn`, `.await`, `Arc<Mutex<>>`, or any async runtime concern. The architecture is inverted, the concurrency is unbounded, the error handling is unsound, and there is zero observability. Reject immediately.

---

## LETHAL FINDINGS (7)

1. **[spawn_at_edge_only] `crates/domain/order_processor.rs:9`** -- `tokio::spawn` inside domain crate code. Spawn is an Action in the Data-Calc-Actions hierarchy and MUST live exclusively at the architecture edge (HTTP handlers, infra adapters, main). Domain logic must be pure, synchronous, and testable without a runtime. Move all spawn logic to the presentation or infrastructure layer. See `references/spawn-discipline.md`.

2. **[no_await_in_calc] `crates/domain/order_processor.rs:1`** -- The entire `process_orders` function is declared `async fn` and lives in the domain crate. The domain crate MUST NOT contain async functions. Domain logic is pure synchronous computation -- it should not await anything. The async shell (presentation/infra layers) calls sync domain functions and awaits infrastructure I/O. See `references/hexagonal-boundaries.md`.

3. **[domain_zero_async_deps] `crates/domain/order_processor.rs:1`** -- This file imports `std::sync::{Arc, Mutex}` and calls `tokio::spawn`, meaning the domain crate must depend on `tokio`. The domain crate MUST NOT depend on tokio, futures, async-std, or any async runtime. Domain must compile and test with zero async dependencies. See `references/hexagonal-boundaries.md`.

4. **[bounded_concurrency] `crates/domain/order_processor.rs:8-15`** -- Unbounded spawn loop: `for order in orders { tokio::spawn(...) }` creates one task per order with no bound. If `orders` contains 100,000 entries, this spawns 100,000 concurrent tasks -- a denial-of-service vector that will OOM the service under load. Use `for_each_concurrent(N, ...)` or `buffer_unordered(N)` with an explicit concurrency bound. See `references/stream-patterns.md`.

5. **[no_imperative_concurrency] `crates/domain/order_processor.rs:8-17`** -- Manual `Vec<JoinHandle>` bookkeeping with `for h in handles { h.await }`. This is an imperative anti-pattern. Use structured concurrency primitives: `tokio::task::JoinSet` or `futures::stream::FuturesUnordered`. JoinSet provides proper cleanup (aborts remaining tasks on drop) and cleaner result collection. See `references/stream-patterns.md`.

6. **[arc_over_rc] `crates/domain/order_processor.rs:3,11,13`** -- `Arc<Mutex<Vec<...>>>` used as shared mutable state across spawned tasks. This serializes access and defeats parallelism. Worse, `std::sync::Mutex` is used, not `tokio::sync::Mutex`, and while the lock is not held across an `.await` point here (the `.lock().unwrap()` / `.push()` is sync), the entire pattern is unnecessary. Use ownership transfer (move results out of each task via return value) or message passing (mpsc channel). The ownership priority ladder was not consulted. See `references/send-sync-ownership.md`.

7. **[cancellation_safe_design] `crates/domain/order_processor.rs:10-14`** -- If any spawned task panics or is cancelled between `save_to_db` (line 12) and the `guard.push(saved)` (line 13), the saved data exists in the database but is lost from the in-memory results collection. The state is inconsistent -- the DB has data that the caller never sees. Design atomic state transitions: either collect results via return values from JoinHandle, or use a channel. See `references/cancellation-safety.md`.

---

## MAJOR FINDINGS (5)

1. **[avoid_arc_mutex_default] `crates/domain/order_processor.rs:3`** -- `Arc<Mutex<Vec<ProcessedOrder>>>` is used as the default concurrency pattern with zero justification. The priority ladder was ignored: (1) ownership transfer would work here since each task produces exactly one result, (2) message passing via mpsc channel would also work, (3) no attempt at atomics or concurrent maps was considered. See `references/send-sync-ownership.md`.

2. **[tracing_instrument] `crates/domain/order_processor.rs:3`** -- The async function `process_orders` has no `#[tracing::instrument]` annotation. Every async function in the shell MUST have instrumentation. Bare `.await` without surrounding span context makes production debugging impossible. See `references/observability-reference.md`.

3. **[span_propagation_in_spawn] `crates/domain/order_processor.rs:9`** -- `tokio::spawn(async move { ... })` spawns a task with no span propagation. The spawned task starts fresh with no trace correlation to the parent. This means you cannot trace an order from request to completion across the spawn boundary. Use `.instrument(tracing::info_span!("process_order", order_id = ...))` on the spawned future. See `references/observability-reference.md`.

4. **[async_error_chain] `crates/domain/order_processor.rs:16`** -- `handle.await.map_err(|_| AppError::TaskFailed)??` uses a closure that discards the JoinError context entirely (`|_| AppError::TaskFailed`). The original error (panic message, cancellation reason) is silently destroyed. Use `.map_err(|e| AppError::TaskFailed(e.to_string()))` or similar to preserve the causal chain. See `references/observability-reference.md`.

5. **[workload_routing] `crates/domain/order_processor.rs:10-11`** -- `validate_order` and `price_order` are synchronous CPU-bound operations mixed into an async context without any consideration of workload routing. If these are CPU-heavy (validation logic, pricing calculations), they should not run on the async runtime. However, since the entire function should not exist in the domain crate in the first place, the correct fix is to extract these as sync domain functions and call them from the shell. See `references/spawn-discipline.md`.

---

## DELEGATED FINDINGS (4)

1. **DELEGATED**: `.unwrap()` at line 13 (`results_clone.lock().unwrap()`) and line 18 (`results.lock().unwrap()`) -- panicked mutex guard. Handled by holzman-rust zero-unwrap rule.

2. **DELEGATED**: `guard.push(saved)` at line 13 -- mutation via `push`. Handled by holzman-rust bounded-mutation rule.

3. **DELEGATED**: `guard.clone()` at line 19 -- unnecessary clone of the entire results vector. Handled by holzman-rust zero-copy/performance rules.

4. **DELEGATED**: Missing type definitions for `Order`, `ProcessedOrder`, `AppError`, `validate_order`, `price_order`, `save_to_db` -- cannot verify domain type purity. Handled by holzman-rust DDD/type rules.

---

## CRITICAL FINDINGS (2)

1. **[bench_async_vs_sync / bench_throughput_baseline]** No benchmark evidence exists for this async hot path. There are no criterion benchmarks comparing sync domain processing vs the async spawned version, no throughput baselines, and no concurrency scaling tests at N=1, N=8, N=64. Performance claims are unverifiable. See `references/benchmark-patterns.md`.

2. **[tokio_console_required]** No tokio-console or console-subscriber configuration is present. The unbounded spawn loop with shared mutex state is exactly the kind of pattern that would show catastrophic busy/scheduled time metrics under load. Without tokio-console, this will be diagnosed by guesswork in production. See `references/observability-reference.md`.

---

## STATIC ANALYSIS

Mode: SNIPPET (no Cargo.toml on disk). All results are static scan predictions, not execution evidence.

### Pattern Scan Results

| Pattern | Location | Match | Severity |
|---------|----------|-------|----------|
| `tokio::spawn` | Line 9 | MATCH | LETHAL: spawn in domain crate |
| `.await` | Lines 12, 16 | MATCH (2 occurrences) | LETHAL: await in domain |
| `Arc::new(Mutex::new(` | Line 3 | MATCH | MAJOR: Arc-Mutex as default |
| `.lock().unwrap()` | Lines 13, 18 | MATCH (2 occurrences) | DELEGATED: unwrap usage |
| `Vec::new()` (unbounded) | Line 3 | MATCH | LETHAL: unbounded concurrency |
| `Vec<JoinHandle>` | Line 7 | MATCH | LETHAL: imperative concurrency |
| `for ... in handles` | Line 16 | MATCH | LETHAL: manual join loop |
| `#[tracing::instrument]` | Absent | NO MATCH | MAJOR: missing instrumentation |
| `.instrument(` | Absent | NO MATCH | MAJOR: missing span propagation |
| `println!` / `eprintln!` | Absent | NO MATCH | OK |
| `Rc<` / `RefCell<` | Absent | NO MATCH | OK |
| `buffer_unordered` | Absent | NO MATCH | Context: should be used instead |
| `for_each_concurrent` | Absent | NO MATCH | Context: should be used instead |
| `JoinSet` | Absent | NO MATCH | Context: should be used instead |

### Predicted Clippy Lint Outcomes

| Lint | Prediction | Reasoning |
|------|-----------|-----------|
| `clippy::await_holding_lock` | WOULD NOT fire | The `MutexGuard` is obtained and dropped within the sync portion of the spawned task (lines 13-14), not held across an `.await`. The `.lock().unwrap()` and `.push()` are sync operations. However, the `.await` at line 12 (`save_to_db`) does not hold the lock. |
| `clippy::unused_async` | WOULD NOT fire | The function does contain `.await` points (lines 12, 16), so it is not "unused" async. |
| `clippy::large_futures` | MIGHT fire | The spawned async block captures `order` (moved), `results_clone` (Arc clone), and contains the full state machine for validate/price/save. Depends on the size of `Order` and `ProcessedOrder`. Worth investigating if Order is large. |
| `clippy::unwrap_used` | WOULD fire | Lines 13 and 18 both use `.lock().unwrap()`. Under holzman-rust rules this is denied. |

### Architecture Boundary Scan (Predicted)

```
SCAN: crates/domain/order_processor.rs for async patterns
  .await          -> MATCH at lines 12, 16   -> FAIL: .await in domain
  tokio::spawn    -> MATCH at line 9          -> FAIL: spawn in domain
  Arc<Mutex<>>    -> MATCH at line 3          -> FAIL: shared mutable state in domain
  async fn        -> MATCH at line 3          -> FAIL: async in domain
Result: domain boundary VIOLATED on all axes
```

---

## MANDATE

This code is REJECTED. The following changes are required, in order:

1. **Extract domain logic to sync functions.** `validate_order` and `price_order` are synchronous domain calculations. They must be plain `fn` in the domain crate with zero async dependencies. No `Arc`, no `Mutex`, no `tokio`. See `references/hexagonal-boundaries.md` for the layer structure.

2. **Move the orchestration to the presentation/infra layer.** The HTTP handler or infrastructure adapter is where `async fn`, `tokio::spawn`, and `.await` belong. The handler calls the sync domain functions and awaits the infrastructure I/O. See `references/hexagonal-boundaries.md` for the adapter_owns_async rule.

3. **Replace the unbounded spawn loop with bounded concurrency.** Use `futures::stream::iter(orders).map(|order| async move { ... }).buffer_unordered(32)` or `for_each_concurrent(32, ...)` with an explicit concurrency bound. Never spawn one task per item without a bound. See `references/stream-patterns.md` for the buffer_unordered and for_each_concurrent examples.

4. **Replace `Arc<Mutex<Vec<>>>` with ownership transfer or channels.** Each task returns its `ProcessedOrder` via the `JoinHandle` return value. Collect results from `JoinSet::join_next()` or `buffer_unordered().collect()`. No shared mutable state needed. See `references/send-sync-ownership.md` for the ownership priority ladder.

5. **Add `#[tracing::instrument]` to the async shell function** and propagate spans into any spawned tasks with `.instrument(info_span!(...))`. See `references/observability-reference.md`.

6. **Preserve error context** -- do not discard `JoinError` with `|_| AppError::TaskFailed`. Map to a meaningful error that preserves the original cause. See `references/observability-reference.md`.

7. **Add criterion benchmarks** for the order processing pipeline at N=1, N=8, N=64 concurrency to verify scaling and establish throughput baselines. See `references/benchmark-patterns.md`.

### Correct Architecture Sketch

```
crates/domain/order.rs (sync, no async deps):
  pub fn validate_order(order: &Order) -> Result<ValidatedOrder, DomainError>
  pub fn price_order(order: &ValidatedOrder) -> PricedOrder

crates/infra/order_repo.rs (async, depends on domain):
  pub async fn save_to_db(order: &PricedOrder) -> Result<SavedOrder, InfraError>

crates/api/handlers.rs (async shell, depends on app + infra):
  #[tracing::instrument(skip(repo))]
  pub async fn process_orders(orders: Vec<Order>, repo: &dyn OrderRepo) -> Result<Vec<ProcessedOrder>, AppError> {
      futures::stream::iter(orders)
          .map(|order| async {
              let validated = validate_order(&order)?;       // sync domain
              let priced = price_order(&validated);          // sync domain
              let saved = save_to_db(&priced).await?;        // async infra
              Ok(saved)
          })
          .buffer_unordered(32)
          .try_collect()
          .await
  }
```
