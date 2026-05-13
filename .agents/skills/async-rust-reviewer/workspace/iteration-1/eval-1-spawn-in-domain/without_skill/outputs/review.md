# Async Rust Code Review: `process_orders`

**File**: `crates/domain/order_processor.rs`
**Date**: 2026-04-03
**Reviewer**: Baseline (no skill)

---

## Summary

This function processes a batch of orders by spawning a Tokio task per order, collecting results into a shared `Arc<Mutex<Vec<_>>>`. It contains several correctness bugs and performance anti-patterns.

---

## Issues Found

### CRITICAL: Correctness

#### 1. Silent Data Loss on Panic (`unwrap` on Mutex)

```rust
let mut guard = results_clone.lock().unwrap();
```

`Mutex::lock().unwrap()` will panic if the thread holding the lock panicked (poisoned mutex). Since this code spawns multiple tasks, if any task panics while holding the lock, all subsequent `.lock().unwrap()` calls across all other spawned tasks will also panic, causing cascading failures. In production async Rust, you should either handle `PoisonError` explicitly or use `lock().unwrap_or_else(|e| e.into_inner())` depending on your recovery strategy. Better yet, avoid the shared mutable state entirely (see issue #4).

#### 2. Double Error Handling / Inconsistent Result Collection

The function collects processed orders into the shared `Vec` **inside** each spawned task, **and** also propagates errors via `handle.await.map_err()??`. This means:
- If a task succeeds, its result is pushed to the shared vec **and** also returned from `handle.await`.
- If a task fails, the error is propagated via the `??` double-question-mark, but partial results from successful tasks are already in the shared vec.

The `Ok(saved)` returned from the task closure is never actually used by the caller -- only the side effect of pushing into the shared vec matters. This is confusing and error-prone. If the intent is to collect all successful results, the `??` on the handle will abort early on the first failure, potentially returning an error even though some results were already collected.

#### 3. `Arc<Mutex<Vec<_>>>` is Unnecessary

Each spawned task pushes a result, then the main function awaits all handles before reading the shared vec. Since all tasks are joined before the final read, you could instead just collect the `JoinHandle` results directly:

```rust
let results: Vec<ProcessedOrder> = futures::future::join_all(handles)
    .await
    .into_iter()
    .collect::<Result<Result<Vec<_>, _>, _>>()???;
```

The shared mutable state is both a correctness hazard and unnecessary given the control flow.

#### 4. Spawned Tasks May Outlive Critical Resources

`tokio::spawn` creates a `'static` task. The function takes `orders: Vec<Order>` by value, which is fine, but the task closure captures `order` by move. However, if `save_to_db`, `validate_order`, or `price_order` reference any non-`'static` state (e.g., a database connection pool that gets dropped), the spawned tasks could reference freed memory. This depends on the signatures of those functions, but the pattern is worth flagging. Using `tokio::spawn` in domain code is inherently dangerous because domain logic should not be coupled to a specific runtime's task spawning strategy.

### HIGH: Performance

#### 5. Unbounded Task Spawning

One Tokio task is spawned per order with no concurrency limit. If `orders` contains 100,000 items, 100,000 tasks are spawned simultaneously. This can exhaust memory, saturate the Tokio runtime, and cause excessive context switching. Use `tokio::sync::Semaphore` or `StreamExt::buffer_unordered` to bound concurrency:

```rust
use futures::stream::{self, StreamExt};
let results: Vec<_> = stream::iter(orders)
    .map(|order| async { process_single_order(order).await })
    .buffer_unordered(64) // limit to 64 concurrent tasks
    .collect()
    .await;
```

#### 6. Lock Contention on Mutex

Every spawned task contends on the same `Mutex` lock. Even though each lock hold is brief (just a `push`), with many concurrent tasks this creates unnecessary contention. `tokio::sync::Mutex` would be even worse here (it's designed for holds across `.await` points, but adds async overhead). The correct solution is to eliminate the shared state entirely and collect results from the `JoinHandle` return values.

#### 7. Sequential Await of Handles

```rust
for handle in handles {
    handle.await.map_err(|_| AppError::TaskFailed)??;
}
```

This awaits handles in insertion order. If task N fails, tasks N+1..M are never awaited, which means:
- They are **detached** and continue running in the background (since `tokio::spawn` was used).
- Their results are lost.
- If they hold resources (DB connections, file handles), those resources leak until the tasks complete.

Use `futures::future::join_all` or `try_join_all` to await all handles concurrently and ensure none are orphaned.

### MEDIUM: Design / Architecture

#### 8. Domain Layer Should Not Spawn Runtime Tasks

This code lives in `crates/domain/`, which should contain pure business logic. `tokio::spawn` is an infrastructure/runtime concern. Spawning tasks in the domain layer:
- Couples the domain to Tokio specifically.
- Makes the function impossible to test without a Tokio runtime.
- Violates the dependency inversion principle.

The domain should expose a pure async function that processes a single order, and the orchestration (spawning, concurrency control) should live in `crates/app/` or `crates/infra/`.

#### 9. No Cancellation Safety Consideration

If the outer function is cancelled (e.g., the caller drops the future), all spawned tasks continue running independently. There is no `CancellationToken` or abort mechanism. This can lead to wasted work or, worse, partial database writes.

#### 10. Mixed Sync and Async Boundaries

```rust
let validated = validate_order(&order)?;  // sync
let priced = price_order(&validated);      // sync
let saved = save_to_db(&priced).await?;    // async
```

If `validate_order` or `price_order` are CPU-intensive, they will block the Tokio runtime thread. These should be wrapped in `tokio::task::spawn_blocking` if they do significant work, or the function should document that it expects these to be lightweight.

---

## Severity Summary

| # | Issue | Severity | Category |
|---|-------|----------|----------|
| 1 | `unwrap` on Mutex lock (cascading panics) | CRITICAL | Correctness |
| 2 | Double error handling / inconsistent collection | CRITICAL | Correctness |
| 3 | `Arc<Mutex<Vec>>` is unnecessary | CRITICAL | Correctness |
| 4 | `'static` task spawning risk | CRITICAL | Correctness |
| 5 | Unbounded task spawning | HIGH | Performance |
| 6 | Lock contention | HIGH | Performance |
| 7 | Sequential await leaks detached tasks | HIGH | Correctness/Performance |
| 8 | Domain layer spawns runtime tasks | MEDIUM | Architecture |
| 9 | No cancellation safety | MEDIUM | Correctness |
| 10 | Blocking sync work on async runtime | MEDIUM | Performance |

---

## Suggested Rewrite

```rust
// crates/app/order_service.rs (orchestration in app layer, not domain)
use futures::stream::{self, StreamExt};

const MAX_CONCURRENCY: usize = 64;

pub async fn process_orders(
    orders: Vec<Order>,
    db: Arc<dyn OrderRepository>, // dependency injection, not static spawn
) -> Result<Vec<ProcessedOrder>, AppError> {
    let results: Vec<Result<ProcessedOrder, AppError>> = stream::iter(orders)
        .map(|order| {
            let db = Arc::clone(&db);
            tokio::spawn(async move {
                let validated = validate_order(&order)?;  // domain pure fn
                let priced = price_order(&validated);      // domain pure fn
                let saved = db.save(&priced).await?;       // infra via trait
                Ok(saved)
            })
        })
        .buffer_unordered(MAX_CONCURRENCY)
        .collect()
        .await
        .into_iter()
        .map(|join_result| join_result.map_err(|_| AppError::TaskFailed)?)
        .collect();

    // Partition into successes and failures
    let (successes, failures): (Vec<_>, Vec<_>) = results
        .into_iter()
        .partition(Result::is_ok);

    if !failures.is_empty() {
        return Err(AppError::PartialFailure {
            succeeded: successes.into_iter().map(Result::unwrap).collect(),
            failed_count: failures.len(),
        });
    }

    successes.into_iter().map(Result::unwrap).collect()
}
```

Key improvements:
- **No `Arc<Mutex<Vec>>`** -- results are collected from task return values.
- **Bounded concurrency** via `buffer_unordered`.
- **Domain stays pure** -- orchestration moved to app layer.
- **Dependency injection** for the database, not a static `save_to_db` call.
- **Graceful partial failure** handling instead of silent data loss.
- **No `unwrap`** on locks or mutexes.
