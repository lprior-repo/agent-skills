# copied-model-disconnected

Production target: `TreeMap::insert`, `Node::insert_into_optional`.

Proof artifact only mentions:

```rust
spec fn model_insert(...) -> ... { ... }
proof fn lemma_model_insert_preserves_order(...) { ... }
```

No production call-site refs or bridge evidence.
