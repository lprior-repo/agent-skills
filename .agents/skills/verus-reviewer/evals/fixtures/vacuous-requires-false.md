# vacuous-requires-false

```rust
exec fn parse(input: &[u8]) -> (r: Packet)
  requires false
  ensures r@ == spec_parse(input@)
{ arbitrary_packet() }
```

No reachable production caller witness.
