# parenthesized-verifier-attribute

```rust
#[verifier(external_body)]
pub fn parse_packet(bytes: &[u8]) -> (p: Packet) ensures p@ == spec_parse(bytes@) { arbitrary() }
#[verifier(external)] mod helper {}
```

Trust scan only searched `#[verifier::external_body]`.
