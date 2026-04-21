# Async Stream Processing Code Review

## Code Under Review

```rust
// crates/infra/src/event_processor.rs
use futures::StreamExt;
use std::rc::Rc;

pub async fn process_event_stream(mut stream: impl Stream<Item = Event> + Unpin) {
    let context = Rc::new(ProcessingContext::default());
    let mut results = Vec::new();

    while let Some(event) = stream.next().await {
        if event.is_relevant() {
            let enriched = context.enrich(event);
            let result = process(enriched).await;
            results.push(result);
        }
    }

    Ok(results)
}
```

---

## Critical Correctness Issues

### 1. Function signature declares `()` return type but body returns `Ok(results)` (COMPILE ERROR)

The function signature says `-> impl Future<Output = ()>` (via `async fn ... ()`), but the body contains `Ok(results)`. This will not compile. The return type must be changed to something like `-> Result<Vec<ProcessResult>, Error>` or the `Ok(results)` must be removed.

**Severity:** Compile-time failure.

### 2. `Rc` is not `Send` -- function cannot be used across await points safely

`Rc` is `!Send`. The `process(enriched).await` call is an await point inside an async function. While `Rc` itself does not live across the await here (it is borrowed but not held across the `.await` if `enrich` consumes it), the `Rc<ProcessingContext>` is created once and held for the entire lifetime of the async function. This means the future produced by `process_event_stream` will be `!Send`, which prevents it from being spawned on a multi-threaded runtime (e.g., `tokio::spawn`).

If this function is ever spawned or used in a context requiring `Send`, it will fail. Use `Arc` instead of `Rc` for async code.

**Severity:** High -- latent bug that surfaces at integration time.

### 3. No error handling on the stream or processing

- The stream type is `Stream<Item = Event>` -- there is no error channel. If the stream can fail, this should be `Stream<Item = Result<Event, E>>` or similar.
- `process(enriched).await` has no error handling. If `process` can fail, errors are silently discarded.
- The `Ok(results)` at the end suggests the author intended error handling, but none exists in the body.

**Severity:** High -- silent data loss on errors.

### 4. `results` collection is unbounded

The `Vec` grows without bound. For a long-running or high-throughput stream, this will consume unbounded memory. There is no batching, no backpressure, and no limit.

**Severity:** High -- OOM risk in production.

---

## Performance Issues

### 5. Sequential processing of all events

Each event is awaited individually inside the loop:

```rust
let result = process(enriched).await;
```

This means events are processed one at a time. If `process` involves I/O (network, disk), this serializes all latency. Consider using buffered concurrency via `stream.filter_map(...).buffered(N)` or `futures::stream::buffered` to process multiple events concurrently.

### 6. No backpressure signaling

The loop eagerly consumes from the stream as fast as possible. If the producer is faster than `process`, memory grows unboundedly (via `results`) and there is no mechanism to slow down ingestion.

### 7. Unnecessary allocation via `Rc`

`Rc::new(ProcessingContext::default())` allocates on the heap. If `ProcessingContext` is small and immutable, it could be passed by value or reference without heap allocation. If it needs shared ownership in async context, use `Arc` (see issue #2).

---

## Style / Minor Issues

### 8. Unused import risk

`use std::rc::Rc;` -- if switched to `Arc`, this becomes dead code.

### 9. `mut stream` but only `.next()` is called

This is fine for `StreamExt::next()`, which requires `&mut self`. Not a bug, but worth noting that the stream is consumed exclusively.

### 10. Missing type annotations / imports

`Event`, `ProcessingContext`, `process`, and `ProcessResult` are all undefined in the snippet. These are presumably defined elsewhere, but the review cannot verify their traits or implementations.

---

## Summary Table

| # | Issue | Severity | Category |
|---|-------|----------|----------|
| 1 | Return type mismatch (`()` vs `Ok(results)`) | **Compile Error** | Correctness |
| 2 | `Rc` makes future `!Send` | **High** | Correctness |
| 3 | No error handling on stream or process | **High** | Correctness |
| 4 | Unbounded `results` collection | **High** | Performance / OOM |
| 5 | Sequential event processing | **Medium** | Performance |
| 6 | No backpressure | **Medium** | Performance |
| 7 | Unnecessary `Rc` allocation | **Low** | Performance |
| 8 | Dead import after fix | **Low** | Style |
| 9 | Stream mutability pattern | **Info** | Style |
| 10 | Missing type context | **Info** | Completeness |

---

## Suggested Rewrite Sketch

```rust
use futures::StreamExt;
use std::sync::Arc;

pub async fn process_event_stream(
    stream: impl Stream<Item = Result<Event, StreamError>> + Unpin,
    concurrency: usize,
) -> Result<Vec<ProcessResult>, ProcessingError> {
    let context = Arc::new(ProcessingContext::default());

    let results: Vec<ProcessResult> = stream
        .filter_map(|event_result| async move {
            match event_result {
                Ok(event) if event.is_relevant() => Some(event),
                Ok(_) => None,
                Err(e) => {
                    tracing::warn!("stream error: {e}");
                    None
                }
            }
        })
        .map(|event| {
            let ctx = context.clone();
            async move {
                let enriched = ctx.enrich(event);
                process(enriched).await
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    Ok(results)
}
```

This addresses issues #1, #2, #3 (partially), #4 (bounded by concurrency), #5, and #6.
