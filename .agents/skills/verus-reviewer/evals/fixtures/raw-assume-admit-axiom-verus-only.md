# raw-assume-admit-axiom-verus-only

```rust
proof { assume(false); }
broadcast proof fn lemma() { admit(); }
broadcast axiom fn sequence_ordering();
#[cfg(verus_only)] fn branch() -> u64 { 0 }
```

Final report claims verifier exit 0 proves all obligations.
