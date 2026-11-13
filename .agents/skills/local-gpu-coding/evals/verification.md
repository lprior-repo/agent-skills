# Verification evidence

## Current installation

Observed with `omp skill list --json`:

- `local-gpu-coding` resolves to `/home/lewis/.agents/skills/local-gpu-coding/SKILL.md`, source `agents:user`, visible.
- `holzman-rust` resolves to `/home/lewis/.agents/skills/holzman-rust/SKILL.md`, source `agents:user`, visible.
- `local-gpu-coding` is a real directory, not a symlink.
- The former managed-skills directory and foreign-harness discovery link are absent.
- No custom OMP agent definitions were added.
- `jq` successfully parsed the three evaluation specifications in `evals.json`; that does not mean those scenarios passed.

## Native worker startup

Both native OMP task probes completed successfully. Their inspected transcripts show an actual successful `read(skill://holzman-rust)` returning 396 lines before the final response:

| GPU | Exact requested model | Transcript evidence |
| --- | --- | --- |
| RTX 5090 | `local-5090-ninfer/qwen3.8-27b-uncensored:high` | `history://Verify5090Skill`, line 11 |
| RTX 3090 | `local-3090-llama/qwen3.8-27b-uncensored:high` | `history://Verify3090Skill`, line 11 |

Both workers performed only the prescribed skill read and reported concrete requirements from its content. Neither edited nor ran Rust code. Transcript session: `2026-11-12T19-32-03-434Z_01a1d13e-e8ea-74a9-8ceb-4e2d7a675d66`.

## Native Rust implementation experiment

Run: [`runs/rust-pair-ntxkUl/results.md`](runs/rust-pair-ntxkUl/results.md).

Both native routes implemented the same dependency-free, `no_std` Rust parser against a coordinator-owned contract and 21 held-out public-API tests. Startup transcripts for `Code5090Selection` and `Code3090Selection` show the required `holzman-rust` read before source creation. Final SHA-256 checks confirm the contract, tests and manifests remained unchanged.

- **5090:** initial compilation failure; three local repair rounds; final 21/21 behavior tests and formatting passed. Strict Clippy passed after one coordinator-only documentation-backtick correction. No coordinator-authored executable Rust fixes.
- **3090:** initial 20/21 tests; first repair failed compilation. The next repair request exhausted 10 retries with an unsupported-image HTTP 500, despite no image blocks in the stored transcript. **Not a passing coding run.** The outgoing request payload was not captured, so the infrastructure cause remains unconfirmed.

Source snapshots, raw gate logs, provider errors and reproducible commands are retained in that run. No other model or harness was substituted. This single-task experiment establishes neither general coding parity nor a throughput advantage.

## Limits

This installation uses an explicit mandatory read in each task packet, not an autoload hook. Tasks that bypass the skill are not mechanically constrained by it. A successful startup probe establishes routing and skill access only, not subsequent adherence, Rust correctness, model parity, or throughput.

Earlier direct-HTTP Rust smoke checks do not verify native OMP task behavior and are not evidence for this installation. No Python validators are part of this workflow.
