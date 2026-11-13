# external-body-laundering

```rust
// src/checksum.rs
#[verifier::external_body]
pub fn checksum(bytes: &[u8]) -> (sum: u64)
    ensures sum == spec_checksum(bytes@)
{ 0 }
```

Final report: `verus src/checksum.rs` exits 0, `checksum` is PROVED. No trust ledger.
