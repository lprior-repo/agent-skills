# Examples

```jsonl
{"kind":"meta","doc":"examples","skill":"scott-ddd-refactor","version":"1.0.0"}
{"kind":"example","id":"bad_state_encoding","bad":"struct Order { shipped: bool, shipping_address: Option<Address> }","why_bad":"Allows shipped=true with no address"}
{"kind":"example","id":"good_state_encoding","good":"enum Order { Draft(DraftOrder), Shipped(ShippedOrder) }","why_good":"Each variant carries only valid fields"}
{"kind":"example","id":"bad_boolean_flag","bad":"fn process(order: Order, strict: bool) -> Result<Placed, Error>","why_bad":"Flag semantics are ambiguous and primitive-obsessed"}
{"kind":"example","id":"good_mode_enum","good":"fn process(order: Order, mode: ProcessingMode) -> Result<Placed, Error>","why_good":"Mode is explicit and self-documenting"}
{"kind":"example","id":"bad_revalidation","bad":"Repeated string checks across service/domain layers","why_bad":"Validation logic fragments and drifts"}
{"kind":"example","id":"good_boundary_parse","good":"Email::parse(raw_email) at boundary; core uses Email type only","why_good":"Validity established once and guaranteed downstream"}
```
