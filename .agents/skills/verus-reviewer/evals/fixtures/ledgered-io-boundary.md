# ledgered-io-boundary

```rust
#[verifier::external_body]
fn print_result(value: u64) { println!("{}", value); }
```

Trust ledger: owner `runtime-io`, reason stdout side effect, scope `print_result` only, expiry next I/O abstraction review, compensating evidence `manual CLI stdout smoke test output artifact://print-smoke`.

Core compute proof does not depend on print result.

Missing from packet: final verifier command output, trust-scan command/output, code-parity chain for compute, and behavior-test output.
