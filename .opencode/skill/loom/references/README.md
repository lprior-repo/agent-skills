# Loom Skill References

## Official Sources

- [Loom crate docs](https://docs.rs/loom/)
- [Loom GitHub repository](https://github.com/tokio-rs/loom)
- [Loom README](https://github.com/tokio-rs/loom/blob/master/README.md)

## Memory Ordering References

- [Rust std::sync::atomic docs](https://doc.rust-lang.org/std/sync/atomic/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)

## Related Testing Approaches

- [Shuttle](https://github.com/awslabs/shuttle) - Randomized concurrency testing with deterministic replay
- [CHESS project](https://www.microsoft.com/en-us/research/project/chess/) - Systematic schedule exploration
- [CDSChecker source](https://github.com/computersforpeace/model-checker) - C/C++ memory-model checker lineage

## Key Papers

- Dynamic Partial Order Reduction - Flanagan and Godefroid, POPL 2005
- [CHESS: A Systematic Testing Tool for Concurrent Software](https://www.microsoft.com/en-us/research/publication/chess-a-systematic-testing-tool-for-concurrent-software/)
- [Iterative Context Bounding for Systematic Testing of Multithreaded Programs](https://www.microsoft.com/en-us/research/publication/iterative-context-bounding-for-systematic-testing-of-multithreaded-programs/)

## Production Usage

- [Tokio Loom CI workflow](https://github.com/tokio-rs/loom/actions/workflows/loom.yml)
