---
name: loom
description: |
  Use when user says "loom", "loom test", "test concurrency with loom",
  "concurrency permutation test", "race condition test", "memory ordering test",
  "lock-free test", or asks how to systematically test concurrent Rust code
  with schedule exploration. Also use for questions about unsafe concurrency,
  waker logic, model checking for threads, or comparing shuttle/loom/stress-testing
  approaches. Do NOT use for general async Rust (use async-rust-reviewer),
  formal verification of concurrent code (use kani/verus), or setting up async
  runtimes.
---

# Loom: Concurrency Permutation Testing for Rust

## What Loom Is

Loom is a **deterministic model checker** for concurrent Rust code. It repeatedly runs a test closure under different thread schedules and memory-model behaviors, using state-space reduction to avoid redundant executions.

**Use Loom for:**
- Custom synchronization primitives
- Lock-free data structures
- Unsafe concurrency code
- Waker logic and async runtime internals
- Small scheduler-like components

**Do NOT use Loom for:**
- Theorem proving (use Kani/Verus)
- Full async runtime modeling (too large)
- Non-deterministic external dependencies (wall clock, network, RNG)

## Mental Model

| Component | Role |
|-----------|------|
| `loom::model` | Exhaustively or boundedly re-runs a closure |
| Loom-aware threads/atomics/sync | Instruments concurrent operations |
| Scheduler explorer | Explores valid schedules; prunes equivalent executions |
| Pass | All explored executions satisfy invariants |

A **passing Loom test is stronger than a passing stress test** because Loom doesn't wait to "get lucky" - it systematically explores the modeled state space.

## Project Setup

### Cargo.toml

```toml
[target.'cfg(loom)'.dependencies]
loom = { version = "0.7", features = ["futures", "checkpoint"] }

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(loom)'] }
```

### Sync Indirection Layer (src/sync.rs)

```rust
#[cfg(loom)]
pub(crate) mod sync {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::{Arc, Condvar, Mutex, RwLock};
    pub(crate) use loom::sync::atomic::{fence, AtomicBool, AtomicUsize, Ordering};
    pub(crate) use loom::thread;
}

#[cfg(not(loom))]
pub(crate) mod sync {
    pub(crate) use std::sync::{Arc, Condvar, Mutex, RwLock};
    pub(crate) use std::sync::atomic::{fence, AtomicBool, AtomicUsize, Ordering};
    pub(crate) use std::thread;

    #[derive(Debug)]
    pub(crate) struct UnsafeCell<T>(std::cell::UnsafeCell<T>);
    impl<T> UnsafeCell<T> {
        pub(crate) fn new(data: T) -> Self { Self(std::cell::UnsafeCell::new(data)) }
        pub(crate) fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R { f(self.0.get()) }
        pub(crate) fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R { f(self.0.get()) }
    }
}
```

**Critical rule:** Library code imports from the local `crate::sync::sync` module - never directly from `std` or `loom`.

## Running Tests

```bash
# Exhaustive (tiny models only)
RUSTFLAGS="--cfg loom" cargo test --test loom_mytest --release

# Bounded smoke (PR runs)
RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=2 LOOM_MAX_BRANCHES=10000 cargo test --test loom_mytest --release

# Bounded deeper (scheduled CI)
RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test --test loom_mytest --release
```

**Always run in `--release`.** Loom re-executes the same closure many times; optimized code is the difference between usable and painful.

## Core API

```rust
use crate::sync::sync::{Arc, AtomicBool, AtomicUsize, Ordering, thread};

// Minimal exhaustive test: two threads incrementing a counter
#[test]
fn two_threads_increment_counter() {
    loom::model(|| {
        let counter = Arc::new(AtomicUsize::new(0));

        let t1 = {
            let counter = counter.clone();
            thread::spawn(move || {
                counter.fetch_add(1, Ordering::AcqRel);
            })
        };

        let t2 = {
            let counter = counter.clone();
            thread::spawn(move || {
                counter.fetch_add(1, Ordering::AcqRel);
            })
        };

        t1.join().unwrap();
        t2.join().unwrap();

        assert_eq!(2, counter.load(Ordering::Relaxed));
    });
}

// Bounded test with Builder
#[test]
fn bounded_publication_protocol() {
    let mut b = loom::model::Builder::new();
    b.max_threads = 3;
    b.preemption_bound = Some(2);

    b.check(|| {
        let data = Arc::new(AtomicUsize::new(0));
        let ready = Arc::new(AtomicBool::new(false));

        let t1 = {
            let data = data.clone();
            let ready = ready.clone();
            thread::spawn(move || {
                data.store(42, Ordering::Relaxed);
                ready.store(true, Ordering::Release);
            })
        };

        let t2 = {
            let data = data.clone();
            let ready = ready.clone();
            thread::spawn(move || {
                if ready.load(Ordering::Acquire) {
                    assert_eq!(42, data.load(Ordering::Relaxed));
                }
            })
        };

        t1.join().unwrap();
        t2.join().unwrap();
    });
}
```

## Memory Ordering Quick Reference

| Pattern | Ordering | When to Use |
|---------|----------|-------------|
| Publication | Release on store, Acquire on load | One-way state publication |
| Read-modify-write | AcqRel | Atomic updates that both observe and publish |
| Simple global order needed | SeqCst | When full sequential mental model is worth the cost |

**Start with Acquire/Release for one-way publication.** Let Loom challenge assumptions.

## Builder Tuning Knobs

| Field | Env Variable | Purpose |
|-------|-------------|---------|
| `max_threads` | none | Keep below MAX_THREADS (=5) |
| `max_branches` | `LOOM_MAX_BRANCHES` | Cap thread switches per permutation |
| `preemption_bound` | `LOOM_MAX_PREEMPTIONS` | Primary scalability lever |
| `max_permutations` | `LOOM_MAX_PERMUTATIONS` | Hard cap on explored permutations |
| `max_duration` | `LOOM_MAX_DURATION` | Time budget |
| `checkpoint_file` | `LOOM_CHECKPOINT_FILE` | Save/load progress |
| `location` | `LOOM_LOCATION` | Capture operation locations (expensive) |
| `log` | `LOOM_LOG` | Print execution logs |

**Bounding priority:** Shrink model first -> add preemption_bound 2-3 -> then permutation/duration caps.

## Spin Loops and yield_now

**Loom's scheduler is NOT fair.** Any spin loop MUST yield:

```rust
fn wait_until_ready(flag: &AtomicBool) {
    while !flag.load(Ordering::Acquire) {
        #[cfg(loom)]
        loom::thread::yield_now();
        #[cfg(not(loom))]
        std::hint::spin_loop();
    }
}
```

## Blocking Operations (Will Hang)

**These operations are NOT modeled by Loom and will cause tests to hang:**

| Operation | Why it hangs | Alternative |
|-----------|-------------|-----|
| `std::thread::sleep` | Not modeled | Remove; test determinism |
| `std::thread::park` | Not modeled | `loom::thread::yield_now()` |
| `std::sync::Barrier` | Unsupported stub | Rewrite without barrier |
| `std::sync::mpsc` | Stub module | Rewrite with Loom-aware types |

Loom only intercepts operations through its own types. Any blocking std call bypasses the model checker.

## Debugging Failures

```bash
# 1. Isolate failing path to checkpoint file
LOOM_CHECKPOINT_FILE=fail.json \
RUSTFLAGS="--cfg loom" cargo test --test loom_mytest --release failing_case

# 2. Make next permutation the failing one
LOOM_CHECKPOINT_INTERVAL=1 \
LOOM_CHECKPOINT_FILE=fail.json \
RUSTFLAGS="--cfg loom" cargo test --test loom_mytest --release failing_case

# 3. Trace that single execution
LOOM_LOG=trace \
LOOM_LOCATION=1 \
LOOM_CHECKPOINT_INTERVAL=1 \
LOOM_CHECKPOINT_FILE=fail.json \
RUSTFLAGS="--cfg loom" cargo test --test loom_mytest --release failing_case
```

**Rule:** Enable `LOOM_LOCATION` only after isolating the failing permutation.

## Unsupported/Stable Primitives

| Primitive | Status |
|-----------|--------|
| `loom::sync::Barrier` | Unsupported stub |
| `loom::sync::mpsc` | Stub module |
| `MAX_THREADS` | Hard limit = 5 |
| `loom::cell::UnsafeCell` | Use `with`/`with_mut` closures |
| `loom::thread::yield_now` | Required in spin loops |
| `loom::sync::atomic::fence(SeqCst)` | Supported |
| SeqCst loads/stores | Treated as AcqRel (not full SeqCst) |

## Loom vs Alternatives

| Approach | Strategy | Pass Meaning |
|----------|----------|--------------|
| **Loom** | Systematic exploration with state reduction | Strong confidence for modeled space |
| **Shuttle** | Randomized scheduling with replay | Useful bug-finding, not proof |
| **Stress tests** | Repeat and hope | Little schedule coverage |

## CI Configuration Template

```yaml
name: loom
on: [pull_request, push, schedule]

jobs:
  loom-compile:
    runs-on: ubuntu-latest
    env:
      RUSTFLAGS: --cfg loom -Dwarnings
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --tests --no-run --release

  loom-smoke:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        test: [loom_queue, loom_notify, loom_async]
    env:
      RUSTFLAGS: --cfg loom
      LOOM_MAX_PREEMPTIONS: 2
      LOOM_MAX_BRANCHES: 10000
      RUST_BACKTRACE: 1
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --test ${{ matrix.test }} --release -- --nocapture
```

## Checklist

- [ ] Test is fully deterministic (no hidden RNG, time, syscalls)
- [ ] All concurrency primitives go through local `sync` indirection
- [ ] Model is tiny and focused on the protocol, not the whole subsystem
- [ ] Busy-wait loops call `yield_now` under `cfg(loom)`
- [ ] Unsafe shared state uses Loom `UnsafeCell` in the model path
- [ ] Start exhaustive on smallest model before adding bounds
- [ ] Use low preemption bounds first (2-3)
- [ ] Debug with checkpointing before enabling location capture
- [ ] Run Loom tests separately and in `--release`
- [ ] Compile-check `cfg(loom)` paths in CI
- [ ] No blocking std operations (`sleep`, `park`) in model

## Response Format

When completing a Loom task, provide:

1. **Complete runnable test code** - no placeholder comments
2. **Exact `RUSTFLAGS="--cfg loom"` command** with any `LOOM_MAX_*` bounds needed
3. **Expected behavior** - what Loom will explore and what a pass/fail means
4. **For failures**: the checkpoint + `LOOM_LOG=trace LOOM_LOCATION=1` debugging workflow

## ANTI-VERIFICATION LAUNDERING MANDATE (LOOM)
AI agents will cheat Loom concurrency models to make them finish quickly or avoid interleaving failures. You MUST actively hunt for and REJECT the following "Verification Laundering" tactics:
1. **State-Space Starvation**: Configuring `loom::model` with `Builder::new().max_branches(1)` or `max_preemptions(0)`. This artificially restricts schedule exploration, defeating the entire purpose of Loom. REJECT.
2. **Missing Assertions**: A Loom test that just runs concurrent threads but asserts nothing at the end proves nothing about data consistency. REJECT.
You MUST ensure Loom models fully explore the preemption space.
