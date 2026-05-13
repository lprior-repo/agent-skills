# Async Rust Code Review

**File**: `crates/domain/order_processor.rs`
**Date**: 2026-04-03
**Reviewer**: async-rust-reviewer (skill v1.0.0)

---

## VERDICT: REJECTED

This code fails at Phase 1 (Spawn Discipline) with lethal findings. It would
also fail at Phases 2, 3, 4, 5, 6, and 7. The review proceeds through all
phases because every phase reveals additional violations.

---

## PHASE 1: Spawn Discipline & Runtime Hygiene

### LETHAL-1: tokio::spawn inside domain crate

**Location**: `crates/domain/order_processor.rs`, line 8 (`tokio::spawn(...)`)

The file path is `crates/domain/order_processor.rs`. Domain crate must have
zero async dependencies. `tokio::spawn` is a runtime primitive. It does not
belong in domain logic -- ever. Spawn is an Action and belongs exclusively in
the Actions layer (handlers, infra adapters, main). This violates rule
`spawn_at_edge_only` (level: fatal).

### LETHAL-2: async fn in domain crate

**Location**: `crates/domain/order_processor.rs`, line 3 (`pub async fn process_orders`)

The function `process_orders` is declared `async` and lives in `crates/domain/`.
Domain logic must be pure, synchronous, and testable without a runtime. The
presence of `async fn` in the domain crate means the domain now depends on an
async runtime. This violates rule `no_await_in_calc` (level: fatal) and
`domain_zero_async_deps` (level: fatal).

### LETHAL-3: CPU-bound work treated as async

**Location**: `crates/domain/order_processor.rs`, lines 10-11

```rust
let validated = validate_order(&order)?;
let priced = price_order(&validated);
```

`validate_order` and `price_order` appear to be synchronous domain calculations
(pure functions with no `.await`). Wrapping them inside `tokio::spawn` and
forcing them onto async task contexts is an anti-pattern. CPU-bound work must
use Rayon or remain synchronous -- not be falsely paralleled via async tasks.
This violates rule `sync_over_async` (level: fatal) and
`workload_routing` (level: error).

### LETHAL-4: Unbounded spawn loop

**Location**: `crates/domain/order_processor.rs`, lines 6-15

```rust
for order in orders {
    let handle = tokio::spawn(async move { ... });
    handles.push(handle);
}
```

This spawns one task per order with no bound on concurrency. If `orders`
contains 100,000 items, 100,000 tasks are spawned simultaneously. This is a
denial-of-service vector that will exhaust memory and scheduler resources under
load. This violates rule `bounded_concurrency` (level: fatal).

---

## PHASE 2: Stream Combinators & Concurrency Primitives

### LETHAL-5: Imperative task bookkeeping with Vec<JoinHandle>

**Location**: `crates/domain/order_processor.rs`, lines 5 and 17-19

```rust
let mut handles = Vec::new();
...
for handle in handles {
    handle.await.map_err(|_| AppError::TaskFailed)??;
}
```

Manual `Vec<JoinHandle>` collection followed by a loop-join is the textbook
anti-pattern. The code must use `tokio::task::JoinSet` or
`futures::stream::FuturesUnordered` or `buffer_unordered(N)` with an explicit
concurrency bound. This violates rule `no_imperative_concurrency` (level: error).

### MAJOR-1: No structured concurrency

The spawned tasks have no lifecycle management. If `process_orders` is
cancelled (e.g., via `select!` or timeout), the spawned tasks continue running
indefinitely. `JoinSet` would abort remaining tasks on drop, providing proper
structured concurrency. The current code leaks work on cancellation.

---

## PHASE 3: Send + Sync Hygiene & Ownership Design

### LETHAL-6: std::sync::Mutex across .await in spawned tasks

**Location**: `crates/domain/order_processor.rs`, lines 13-14

```rust
let mut guard = results_clone.lock().unwrap();
guard.push(saved);
```

Inside the spawned task, `results_clone.lock().unwrap()` acquires a
`std::sync::Mutex`. While the `.unwrap()` is not technically across an `.await`
here, this pattern is catastrophically fragile: the guard is held while `saved`
(which may involve complex Clone logic) is pushed. More critically, this same
`Arc<Mutex<Vec<...>>>` is accessed from the outer function at line 21:

```rust
let guard = results.lock().unwrap();
```

Using `std::sync::Mutex` in async contexts where the lock guard exists in the
same scope as `.await` points (the spawned tasks do contain `.await` at
`save_to_db`) is a clippy violation (`clippy::await_holding_lock`). This
violates rule `arc_over_rc` (level: fatal).

### LETHAL-7: .unwrap() on Mutex lock -- panics kill the runtime

**Location**: `crates/domain/order_processor.rs`, lines 13 and 21

```rust
let mut guard = results_clone.lock().unwrap();  // line 13
let guard = results.lock().unwrap();             // line 21
```

`.unwrap()` on a `Mutex::lock()` call will panic if the Mutex is poisoned (i.e.,
a previous holder panicked). In an async context, a panic in a spawned task
poisons the Mutex, and then every subsequent `.lock().unwrap()` panics as well,
causing a cascade failure across the entire runtime. This violates the
holzman-rust zero-panic mandate and rule `arc_over_rc`.

### MAJOR-2: Arc<Mutex<Vec<...>>> without priority ladder justification

**Location**: `crates/domain/order_processor.rs`, line 4

```rust
let results = Arc::new(Mutex::new(Vec::new()));
```

This is the first-resort pattern, not the last. The priority ladder demands:

1. **Ownership transfer** -- each task could return its result, no sharing needed.
2. **Message passing** -- tasks could send results through an `mpsc` channel.
3. **FuturesUnordered/buffer_unordered** -- collect results naturally from the stream.

None of these alternatives were considered. `Arc<Mutex<Vec<...>>>` serializes
access and defeats the parallelism the code is trying to achieve. This violates
rule `avoid_arc_mutex_default` (level: error).

---

## PHASE 4: Cancellation Safety & Pin Awareness

### LETHAL-8: Cancellation-unsafe state mutation

**Location**: `crates/domain/order_processor.rs`, lines 11-13

```rust
let saved = save_to_db(&priced).await?;        // .await point
let mut guard = results_clone.lock().unwrap(); // state mutation after await
guard.push(saved);
```

If the spawned task's Future is dropped (cancelled) between `save_to_db` and
the `push`, the database write has committed but the result is lost from the
in-memory collection. This creates a state inconsistency: the database has the
order, but the returned `Vec<ProcessedOrder>` does not contain it. The caller
has no way to know which orders were actually persisted.

Additionally, `save_to_db` itself is a stateful `.await` -- if cancelled during
the database write, the operation may be partially committed depending on the
database driver's transaction semantics. This violates rule
`cancellation_safe_design` (level: fatal).

---

## PHASE 5: Observability & Error Propagation

### MAJOR-3: No tracing instrumentation

**Location**: `crates/domain/order_processor.rs`, line 3

The async function `process_orders` has no `#[tracing::instrument]` attribute.
Inside the spawned tasks, there are no span creations, no `tracing::info!`,
`tracing::debug!`, or `tracing::error!` calls. If any task fails in production,
there is zero observability into what happened, which order caused the failure,
or how long each step took. This violates rule `tracing_instrument` (level: error).

### MAJOR-4: No span propagation into spawned tasks

**Location**: `crates/domain/order_processor.rs`, line 8

`tokio::spawn` is called without `.instrument(tracing::info_span!(...))`. Spawned
tasks inherit no trace context. In production, you cannot correlate a failed
order processing task back to the request that triggered it. This violates rule
`span_propagation_in_spawn` (level: error).

### MAJOR-5: Error context lost in double-? operator

**Location**: `crates/domain/order_processor.rs`, line 18

```rust
handle.await.map_err(|_| AppError::TaskFailed)??;
```

The `??` is a double-`?` operator. The inner `?` unwraps the task's `Result`
(if it succeeded with `Ok(Ok(saved))` or `Ok(Err(e))`), and the outer `?`
handles the `Result<Result<...>, JoinError>`. But the `map_err(|_| AppError::TaskFailed)`
discards the `JoinError` entirely -- no message, no cause chain, no way to
diagnose why the task failed (was it a panic? was it cancelled?). This violates
rule `async_error_chain` (level: error).

### MAJOR-6: Silently discarding results from spawned tasks

**Location**: `crates/domain/order_processor.rs`, lines 13-14

The spawned task computes `saved`, pushes it into the shared `results` Vec, and
then returns `Ok(saved)` via the JoinHandle. But the outer code at line 18 calls
`handle.await.map_err(...)??` which evaluates to `Ok(saved)` -- and this value
is immediately discarded (not bound to a variable). The only result collection
happens through the `Arc<Mutex<Vec<...>>>`. The return value from the spawned
task is wasted, making the code confusing and the ownership model unclear.

---

## PHASE 6: Hexagonal Architecture Boundaries

### LETHAL-9: Domain crate depends on tokio (implicit)

**Location**: `crates/domain/order_processor.rs`, lines 1 and 8

The file imports `std::sync::{Arc, Mutex}` and uses `tokio::spawn`. The path
`crates/domain/order_processor.rs` places this file in the domain crate. Even
if `tokio` were not explicitly imported (it's referenced via full path
`tokio::spawn`), the domain crate's `Cargo.toml` would need `tokio` as a
dependency for this to compile. Domain must have ZERO async dependencies:
no tokio, no futures, no async-std, no async-trait. This violates rule
`domain_zero_async_deps` (level: fatal).

### MAJOR-7: save_to_db called from domain

**Location**: `crates/domain/order_processor.rs`, line 11

```rust
let saved = save_to_db(&priced).await?;
```

`save_to_db` is an infrastructure concern (database persistence). Calling it
from domain logic breaks the hexagonal boundary. The domain should not know
about databases. Persistence must be delegated to an infrastructure adapter
via a trait (port). This violates rule `adapter_owns_async` (level: error).

---

## PHASE 7: Ruthless Async Simplicity

### MAJOR-8: The Async Sniff Test -- this should be sync

**Location**: Entire function, lines 3-23

The async sniff test: Would a sync version of this code be simpler and correct?
Yes. The entire function could be:

```
fn process_orders(orders: Vec<Order>) -> Result<Vec<ProcessedOrder>, AppError> {
    orders.into_iter()
        .map(|order| {
            let validated = validate_order(&order)?;
            let priced = price_order(&validated);
            Ok(priced)
        })
        .collect()
}
```

And then at the infrastructure/handler layer, the persistence (`save_to_db`)
would be done concurrently with proper bounds using `buffer_unordered(N)` or
`for_each_concurrent(N, ...)`. The async here is unjustified for the domain
logic portion. This violates rule `no_unnecessary_async` (level: error).

### MAJOR-9: Function exceeds reasonable .await density

**Location**: `crates/domain/order_processor.rs`

The function body involves `.await` inside spawned tasks (`save_to_db.await`),
`.await` on join handles (`handle.await`), and mutex manipulation -- all in a
19-line function that mixes domain logic, infrastructure calls, concurrency
management, and error handling. This is a single function doing the work of
four separate layers. This violates rule `max_await_points` (level: error).

---

## PHASE-AGNOSTIC: Holzman Rust Sync Core Violations

These are delegated to holzman-rust but observed here:

- **MAJOR-10**: `.unwrap()` on `Mutex::lock()` at lines 13 and 21 -- zero-panic
  mandate violated.
- **MAJOR-11**: `mut guard` at line 13 -- mutable binding for Mutex guard is
  necessary for the push, but the entire `Arc<Mutex<Vec>>` pattern could be
  eliminated if tasks simply returned their results.

---

## EXECUTION EVIDENCE

Since this is hypothetical code (not in a real project on disk), the verification
gate commands cannot be run against a real codebase. However, the following
determinations are made based on static analysis:

### Layer 1: Async Clippy Lints

```
PREDICTED: FAIL
- clippy::await_holding_lock would fire on the Mutex guard in spawned tasks
- clippy::unused_async would NOT fire (the function does contain .await usage)
- clippy::large_futures may fire depending on Order/ProcessedOrder size
```

### Layer 2: Domain Crate Dependencies

```
PREDICTED: FAIL
The file at crates/domain/order_processor.rs uses tokio::spawn, which requires
tokio in Cargo.toml. The scan would report:
  "FAIL: async dependency in domain crate"
```

### Layer 3: No .await or spawn in domain source

```
PREDICTED: FAIL
grep -rn "\.await" crates/domain/order_processor.rs -> MATCH at line 11 (.save_to_db) and line 18 (handle.await)
grep -rn "tokio::spawn" crates/domain/order_processor.rs -> MATCH at line 8
```

### Layer 4: Functional-rust sync gate

```
PREDICTED: FAIL
- .unwrap() used at lines 13, 21 -> clippy::unwrap_used violation
- mutable guard at line 13 -> mut usage
```

### Layer 5: Tests

```
SKIPPED: No test artifacts available for hypothetical code
```

### Layer 6: Benchmarks

```
CRITICAL: No benchmarks exist for async hot paths.
No cargo bench output available. Performance claims about concurrent order
processing are unverifiable.
```

### Layer 7: tokio-console

```
WARN: tokio-console not configured (hypothetical code, no Cargo.toml to check)
```

---

## FINDINGS SUMMARY

| Severity | Count | IDs |
|----------|-------|-----|
| LETHAL   | 9     | LETHAL-1 through LETHAL-9 |
| MAJOR    | 11    | MAJOR-1 through MAJOR-11 |
| CRITICAL | 1     | No benchmarks for async hot paths |

---

## MANDATE (Required Changes Before Re-Review)

The code must be restructured. The following changes are required, in order:

1. **Move order processing to the sync domain layer.** `validate_order` and
   `price_order` are pure calculations. They must remain synchronous functions
   in the domain crate with no async dependencies. Return `Result<Vec<ProcessedOrder>, AppError>` via a simple `Iterator::map().collect()` pipeline.

2. **Move persistence to the infrastructure layer.** `save_to_db` is an
   infrastructure concern. Define a trait (port) in the domain or app crate,
   implement it in `crates/infra/`. The domain never calls the database directly.

3. **Move concurrency orchestration to the presentation/edge layer.** If
   concurrent database writes are needed, place the `tokio::spawn` or
   `buffer_unordered(N)` call in the HTTP handler or a dedicated service in
   `crates/api/` or `crates/app/`.

4. **Use bounded concurrency.** Replace the unbounded spawn loop with
   `futures::stream::iter(orders).map(|o| save(o)).buffer_unordered(32)` or
   `for_each_concurrent(32, ...)` with an explicit concurrency bound.

5. **Replace `Arc<Mutex<Vec<...>>>` with ownership transfer.** Each task returns
   its `ProcessedOrder`. Collect results via `buffer_unordered` or `JoinSet`
   which naturally gather results without shared mutable state.

6. **Replace `std::sync::Mutex` with `tokio::sync::Mutex` if locking across
   `.await` is ever necessary** (it should not be after the above changes).

7. **Eliminate all `.unwrap()` calls.** Use `Result`-propagating lock
   operations or pattern matching.

8. **Add `#[tracing::instrument]` to the handler function** and propagate
   spans into any spawned tasks with `.instrument()`.

9. **Preserve error context.** Replace `map_err(|_| AppError::TaskFailed)`
   with `map_err(|e| AppError::TaskFailed { source: e })` or use `.context()`.

10. **Add criterion benchmarks** for the concurrent order processing pipeline
    at N=1, N=8, N=64 concurrency levels to verify scaling and establish
    throughput baselines.

---

*End of review. This code must not ship.*
