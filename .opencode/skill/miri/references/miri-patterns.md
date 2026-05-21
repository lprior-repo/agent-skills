# Miri Patterns And Examples

These examples are diagnostic patterns, not proof templates. Run them under the local nightly before relying on exact output wording.

## Use-After-Free

```rust
#[test]
fn detects_use_after_free() {
    let p = Box::into_raw(Box::new(123_u32));

    unsafe {
        drop(Box::from_raw(p));
        let _ = *p;
    }
}
```

Expected category: definite UB. Pair the report with `-Zmiri-track-alloc-id=<alloc-id>` when Miri names an allocation ID.

## Intrinsic Precondition Failure

```rust
#[test]
fn overlapping_copy_nonoverlapping() {
    let mut buf = [1_u8, 2, 3, 4];
    let p = buf.as_mut_ptr();

    unsafe {
        std::ptr::copy_nonoverlapping(p, p.add(1), 3);
    }
}
```

Expected category: violated intrinsic contract. Native execution may appear fine; the program is still wrong because `copy_nonoverlapping` requires non-overlap.

## Uninitialized And Invalid Values

```rust
#[test]
fn uninitialized_float_is_invalid() {
    let x: f32 = unsafe { std::mem::MaybeUninit::<f32>::uninit().assume_init() };
    let _ = x + 1.0;
}

#[test]
fn invalid_bool_value() {
    let b: bool = unsafe { std::mem::transmute::<u8, bool>(2) };
    std::hint::black_box(b);
}
```

Expected category: invalid typed value or invalid use of uninitialized data. Do not confuse this with ordinary integer overflow.

## Shared Reference Then Raw Write

```rust
#[test]
fn shared_reference_then_raw_write() {
    let mut x = 0_u32;

    let raw = &mut x as *mut u32;
    let shared = &x;

    unsafe { *raw = 1; }
    std::hint::black_box(*shared);
}
```

Expected category: aliasing violation under the active borrow model. A live shared reference promises no mutation except through `UnsafeCell`.

Debugging ladder:

1. Re-run with `MIRIFLAGS="-Zmiri-backtrace=full"`.
2. Track any named pointer tag with `-Zmiri-track-pointer-tag=<tag>`.
3. Compare with `-Zmiri-tree-borrows` only after recording the default result.
4. Fix the reference/raw-pointer design instead of trying to silence the model.

## Strict Provenance Pointer Tagging

```rust
use core::ptr::addr_of_mut;

fn main() {
    let mut x = 17_u32;

    let p = addr_of_mut!(x);
    let tagged = p.map_addr(|addr| addr | 1);
    let restored = tagged.map_addr(|addr| addr & !1);

    unsafe { *restored = 42; }
    assert_eq!(x, 42);
}
```

Preferred pattern: preserve provenance with `addr_of_mut!` and `map_addr`. Avoid pointer-to-`usize`-to-pointer round trips unless Exposed Provenance is explicitly unavoidable.

## Ordinary Integer Overflow Is Not Rust UB

```rust
fn main() {
    let x: u8 = 255;
    let _ = x + 1;
}
```

Expected category: panic or wrapping behavior depending on overflow checks and build mode, not an invented UB claim. Miri follows Rust's configured overflow behavior.

## Unsupported FFI Is Not Automatically UB

```rust
unsafe extern "C" {
    fn puts(s: *const std::ffi::c_char) -> i32;
}

fn main() {
    unsafe {
        puts(b"hello from ffi\0".as_ptr().cast());
    }
}
```

Expected category: likely unsupported operation unless current Miri and flags model it. Route native FFI behavior to integration tests, sanitizers, or Valgrind. If using an experimental native-library bypass, report it as a waiver.

## Racy Static And Weak Memory

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

static READY: AtomicBool = AtomicBool::new(false);
static mut DATA: usize = 0;

fn main() {
    let writer = thread::spawn(|| {
        unsafe { DATA = 42; }
        READY.store(true, Ordering::Relaxed);
    });

    let reader = thread::spawn(|| {
        while !READY.load(Ordering::Relaxed) {}
        let _ = unsafe { DATA };
    });

    writer.join().unwrap();
    reader.join().unwrap();
}
```

Expected category: data-race or weak-memory-sensitive bug. Use `-Zmiri-many-seeds=0..16` and `-Zmiri-track-weak-memory-loads`. For real lock-free algorithms, also use Loom, Shuttle, Stateright, stress tests, or model checking.

## Diagnostic Categories

Definite UB:

- Dangling pointer dereference.
- Misaligned load or store.
- Invalid typed value.
- Uninitialized read.
- Data race.
- Intrinsic precondition violation.

Model-sensitive UB:

- Stacked Borrows rejection.
- Tree Borrows difference.
- Strict Provenance warning or failure.
- Weak-memory schedule-dependent report.

Unsupported operation:

- Unsupported foreign call.
- Unsupported syscall, networking, or platform API.
- Inline assembly or native library behavior Miri cannot inspect.

Debugging aid:

- Backtrace pruning.
- Allocation or pointer-tag tracking.
- Progress reports.
- Seed stabilization.

## Anti-Patterns

Do not say:

- "Miri passed, so the unsafe abstraction is sound."
- "Miri failed, but native tests pass, so it is fine."
- "Use `cfg(miri)` to replace the unsafe code with a safe fake and claim coverage."
- "Use `-Zmiri-permissive-provenance` as the final fix."
- "Unsupported FFI means the program has UB."
- "One seed explored concurrency enough."

Do say:

- "Miri found no UB for this exact command, target, flags, tests, and seed range."
- "This diagnostic is a Stacked Borrows provenance/aliasing finding; Tree Borrows comparison is additional evidence, not final law."
- "This integration path is unsupported by Miri and requires sanitizer/native-test coverage."
- "The Miri path skips these tests, so this is coverage debt."
