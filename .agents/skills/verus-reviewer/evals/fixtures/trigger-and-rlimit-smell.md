# trigger-and-rlimit-smell

```rust
#[verifier::rlimit(2000)]
mod proof {
  invariant forall|i:int| #[trigger] nonnegative(i) ==> s[i] >= 0
}
```

Use sites mention `s[i]`, not `nonnegative(i)`. No quantifier profile or local split.
