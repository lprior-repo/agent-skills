# Lane Command Templates

Use exact commands from `proof-obligations.planned.jsonl` first. Typical smoke commands: TLC tiny config, Verus target check, `cargo kani --harness`, `cargo flux`, `RUSTFLAGS="--cfg loom" cargo test <model>`, `cargo +nightly miri test <name>`, `cargo test <proptest>`, `cargo fuzz run <target> -- -runs=1000`.
