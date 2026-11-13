# cfg-attr-verifier-trust-marker

```rust
#[cfg_attr(verus_keep_ghost, verifier::external_body)]
fn hash_block(bytes: &[u8]) -> u64 { 0 }

#[cfg_attr(verus_keep_ghost, verifier::external)]
mod ffi_hash {}
```

The submitted regex scan reported no trust markers and the report claims hash behavior PROVED.
