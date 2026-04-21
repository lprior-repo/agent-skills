# Code Review: Async Rust Order Service

## Overall Assessment

The code demonstrates a solid grasp of type-state modeling for order lifecycle transitions. The use of `Draft -> Validated -> Priced` as distinct types within an enum is a good pattern for enforcing domain invariants at compile time. Below are findings organized by severity.

---

## Critical Issues

### 1. `Draft` variant construction does not go through `Order::Draft` correctly

In `handle_create`, the line:

```rust
let draft = Order::Draft { customer_id: cmd.customer_id, items: cmd.items };
```

This uses struct literal syntax on an enum variant, which is valid in Rust (since `Order::Draft` is a tuple-variant-like path to `Draft`). However, the actual type of `draft` here is `Draft`, not `Order`. Then `validate_order` takes a `Draft` and returns `Result<Validated, DomainError>`, which is correct. The real issue is the next line:

```rust
repo.save(&Order::Validated(validated)).await?;
```

`validated` is of type `Validated`, and `Order::Validated(validated)` constructs the enum variant. This works, but it means the `Order` enum is only being used as a wrapper at the boundary of `repo.save`. The internal logic operates on the inner types directly, which is fine, but worth noting as a design consideration (see Design section below).

### 2. `handle_create` uses `?` on `validate_order` but `DomainError` must convert to `AppError`

The line:

```rust
let validated = validate_order(draft)?;
```

The `?` operator requires `DomainError` to implement `Into<AppError>`. This is not shown in the provided code. If this conversion is not implemented, this will not compile. This is not necessarily a bug if the conversion exists elsewhere, but it is a correctness concern that should be verified.

---

## Moderate Issues

### 3. No `Debug`, `Clone`, ` PartialEq` derives on domain types

`OrderId`, `CustomerId`, `Draft`, `Validated`, `Priced`, and `Order` all lack common derives. At minimum `Debug` is needed for tracing/logging, and `Clone` is commonly needed in async contexts where data may need to be cloned across task boundaries. This will cause friction in practice.

### 4. `OrderId` and `CustomerId` are newtype wrappers with no validation

Both `OrderId(String)` and `CustomerId(String)` accept any string, including empty strings. The constructor (not shown) should enforce invariants (non-empty, valid format, etc.). If `cmd.customer_id` comes directly from user input without validation, this is a gap.

### 5. `Order` enum does not encode a `Priced` transition

There is a `Priced` variant but no `price_order` function. This is presumably not yet implemented. The type-state model suggests this is incomplete. No critical issue, but the model is currently half-built.

### 6. Items are `String` without domain meaning

`pub items: smallvec::SmallVec<[String; 4]>` stores item identifiers as bare strings. This should be a proper type like `ItemId` or `OrderLine` with quantity and SKU. Using raw strings loses type safety.

---

## Minor Issues

### 7. `OrderRepo::save` takes `&Order` but may need owned or specific variant

The trait method `async fn save(&self, order: &Order) -> Result<(), DomainError>` takes a reference to the full `Order` enum. If the repo only needs to persist `Validated` or `Priced` orders, the signature could be tightened to accept the specific variant. As written, the repo must handle all variants internally, which widens the API surface.

### 8. `tracing::instrument` attribute placement

```rust
#[tracing::instrument(skip(repo), fields(user_id = %cmd.customer_id))]
```

The `fields(user_id = %cmd.customer_id)` syntax in the attribute captures the value at the start of the span. If `customer_id` does not implement `Display`, this will fail to compile. Given that `CustomerId` is a newtype around `String`, it likely does not implement `Display` unless explicitly derived. Verify this.

### 9. `smallvec` dependency for a domain model

Using `smallvec::SmallVec` in the domain types introduces an implementation detail (stack-optimized vector) into the domain layer. This is a performance optimization that leaks into the type signature. If the domain layer is meant to be pure, consider using `Vec` or a custom collection type and applying the optimization at the infrastructure boundary.

---

## Design Suggestions

### 10. Type-state pattern is partially applied

The `Order` enum wraps `Draft`, `Validated`, and `Priced`, but `validate_order` takes `Draft` directly and returns `Validated` directly, bypassing the `Order` enum. Consider two cleaner approaches:

**Option A: Fully typed transitions**

```rust
impl Draft {
    pub fn validate(self) -> Result<Validated, DomainError> { ... }
}
```

This makes the transition a method on the source type, keeping the state machine self-documenting.

**Option B: Order owns the transitions**

```rust
impl Order {
    pub fn validate(self) -> Result<Order, DomainError> {
        match self {
            Order::Draft(d) => Ok(Order::Validated(d.validate()?)),
            _ => Err(DomainError::InvalidState),
        }
    }
}
```

This keeps `Order` as the single entry point but requires runtime matching.

The current code sits between these two approaches, which can confuse readers about where the state machine logic lives.

### 11. Error type design

`DomainError` and `AppError` are referenced but not defined in the snippet. For an async service, consider:

- `DomainError` should be specific (e.g., `NoItems`, `InvalidCustomerId`, `InvalidState`).
- `AppError` should wrap `DomainError` and add infrastructure concerns (e.g., `Database`, `Serialization`, `Timeout`).
- Both should implement `std::error::Error` and have clear `Display` implementations.

### 12. Async trait consideration

`OrderRepo` uses `async fn` in a trait. This requires either the `async fn in trait` feature (stabilized in Rust 1.75) or the `async-trait` crate. If targeting Rust 1.75+, this is fine. If supporting older compilers, this will need `#[async_trait]`. Verify the MSRV.

---

## Summary

| Category | Count |
|----------|-------|
| Critical | 2 |
| Moderate | 4 |
| Minor | 3 |
| Design | 3 |

The core type-state pattern is sound and the code is readable. The main risks are around error conversion correctness (item 2), missing derives that will cause practical friction (item 3), and the domain model being incomplete (items 5, 6). The type-state pattern would benefit from being applied more consistently (item 10).
