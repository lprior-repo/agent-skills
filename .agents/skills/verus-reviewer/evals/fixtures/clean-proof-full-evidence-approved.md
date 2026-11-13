# clean-proof-full-evidence-approved

## Production source

```rust
// src/ring_buffer.rs:40-52
pub fn push(&mut self, value: u64) -> Result<(), PushError>
    ensures self@ == old(self)@.push(value)
{
    if self.is_full() { return Err(PushError::Full); }
    self.buf[self.tail] = value;
    self.tail = (self.tail + 1) % self.capacity;
    self.len += 1;
    Ok(())
}
```

## Proof artifact

```rust
// proofs/ring_buffer.rs:88-126
proof fn lemma_push_preserves_order(pre: RingView, value: u64)
    requires !pre.full(), pre.capacity > 0
    ensures pre.push(value).seq() == pre.seq().push(value)
{
    // local invariant proof; no trust markers
}
```

## Cargo-Verus opt-in

```toml
# Cargo.toml:11-13
[package.metadata.verus]
verify = true
```

## Raw verifier output

```text
$ cargo verus verify -p ring -- --no-cheating
verus 0.2026.06.30
verification results:: 18 verified, 0 errors
exit status: 0
```

## Trust scan output

```text
$ host-search canonical-trust-marker-pattern --scope changed src/ring_buffer.rs proofs/ring_buffer.rs Cargo.toml
<no matches>
exit status: 0

$ host-search canonical-trust-marker-pattern --scope verifier-closure src/ring_buffer.rs proofs/ring_buffer.rs Cargo.toml crate::ring_spec crate::ring_proof_helpers vstd::prelude vstd::seq
<no matches>
exit status: 0
```

## Parser-safe delta

```text
symbol: RingBuffer::push
requires: unchanged
ensures: unchanged
proof-only loop invariant in lemma body: strengthened internal bound tail < capacity to match existing index proof
attributes: no verifier trust markers
exec body semantics: unchanged
```

## Behavior tests

```text
$ cargo test -p ring push_full push_empty
push_full ... ok
push_empty ... ok
exit status: 0
```

## Review grade claim

```text
claimed_grade: ACCEPTABLE
production_usable: not claimed; full repo CI, fuzz/property, benchmark, and maintainer review evidence is outside this packet
upstream_grade: not claimed; upstream maintainer/domain-owner review is outside this packet
```
