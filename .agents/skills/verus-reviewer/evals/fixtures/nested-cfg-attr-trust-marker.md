# nested-cfg-attr-trust-marker

## Production source

```rust
// src/hash.rs:70-82
#[cfg_attr(any(verus_keep_ghost, feature = "proof"), verifier::external_body)]
pub fn digest(bytes: &[u8]) -> (out: u64)
    ensures out == spec_digest(bytes@)
{
    runtime_digest(bytes)
}
```

## Proof report

```text
$ cargo verus verify -p hash -- --no-cheating
verification results:: 12 verified, 0 errors
exit status: 0

$ rg -n '#\s*\[\s*verifier' src/hash.rs proofs/hash.rs
<no matches>
exit status: 1
```

## Final claim

The report marks `digest` as PROVED and says the trust scan is clean. No trust ledger row is present for `src/hash.rs:70`, and no parser/token-tree scan expands nested `cfg_attr(any(...), verifier::external_body)`.
