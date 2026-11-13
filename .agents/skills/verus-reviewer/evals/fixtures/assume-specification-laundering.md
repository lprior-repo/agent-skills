# assume-specification-laundering

```rust
pub assume_specification<T>[ std::mem::swap::<T> ](a: &mut T, b: &mut T)
    ensures a@ == old(b)@ && b@ == old(a)@;
external_trait_specification! { trait Serialize { ... } }
```

No trust ledger; core proof claims PROVED.
