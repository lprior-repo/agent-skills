# remaining-modern-trust-markers

```rust
#[verifier::assume_termination]
proof fn helper() {}
external_type_specification!(std::sync::Arc<T>);
external_trait_extension!(Serialize);
#[verus_verify(external_body)] fn h() {}
#[verus_verify(external)] mod m {}
```
