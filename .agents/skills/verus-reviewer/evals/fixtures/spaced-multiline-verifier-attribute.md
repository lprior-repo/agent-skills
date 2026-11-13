# spaced-multiline-verifier-attribute

```rust
#[verifier :: external_body]
fn parse_header(bytes: &[u8]) -> Header { arbitrary_header() }

#[verifier (external)]
mod parser_ffi {}

#[verifier(
    external_body
)]
fn parse_payload(bytes: &[u8]) -> Payload { arbitrary_payload() }
```

The line regex scan reported no matches and the report marks parser behavior PROVED.
