# Local Rust discovery seeds

Observed under `/home/lewis/src`. Re-discover paths and manifests before use: these repositories are actively changing. This is a selective map, not a complete inventory or a correctness endorsement. Prefer the target repository's own idioms first.

| Repository | Observed capability | Narrow starting points |
| --- | --- | --- |
| `velvet-ballistics` | Rust 2024 typed compiler/runtime/storage workspace; Fjall pinned to 3.1.4; proptest and fuzz dependencies | `crates/vb_expr/src/expr_parser/`, `expr_lexer/`, `expr_property_tests/`; `crates/vb_storage/src/batch/`, `journal/`, `tests/`; inspect the relevant crate manifest |
| `target-reaper` | Rust 2024 CLI, filesystem/Git/SSH operations; Tokio, Rayon, crossbeam, clap; CLI integration tests | `src/native_git_ssh.rs`, `src/parallel_walk.rs`, `src/lease_store/`; corresponding files under `tests/` |
| `track-scrapper-v2` | Rust 2024 ingestion pipeline; Tokio, reqwest, PostgreSQL SQLx; bounded browser actors and cancellation tests | `src/browser/interception.rs`, `src/browser/native.rs`, `src/browser/review_tests.rs`, `src/transport/`, `src/database/` |
| `clarity` | Workspace using Dioxus 0.7.3, Tokio, Fjall and SQLx | `clarity-web/Cargo.toml`, `clarity-web/src/`, `clarity-web/tests/`; discover the relevant component or domain module before reading |
| `titania` | Rust 2024 workspace for policy/checking and custom linting; strict lint configuration | Resolve members `titania-core`, `titania-policy`, `titania-check`, `titania-dylint` from the workspace manifest, then discover files in the selected crate |
| `titan-knight` | Rust 2024 game workspace split into core, GDExtension and headless crates | Resolve members `tk-core`, `tk-gdext`, `tk-headless` from the workspace manifest; inspect each manifest before assuming engine/runtime APIs |

Do not treat `remote-source-rw` as a distinct architecture merely because of its directory name: its inspected manifest also names the package `target-reaper`. Likewise, `velvet-split`, `isolated/` and `isoloated/` may contain related or experimental worktrees. Exclude them by default; use one only when its exact revision or behavior is relevant and ownership is clear. Do not recursively index all worktrees.

## Inspected example pairs

These locations were read, not executed or certified:

- Async cancellation: `track-scrapper-v2/src/browser/review_tests.rs`, `given_external_fetch_cancellation_when_caller_drops_then_owned_tasks_pages_and_connections_are_drained` (observed lines 715–742). It deliberately drops an in-flight fetch, drains owned tasks, waits for idle connections, and checks pages/connection state. Related bounded channels were found in `src/browser/interception.rs` and `src/browser/native.rs`. Browser/runtime requirements must be checked before reusing or running this test.
- Storage: `velvet-ballistics/crates/vb_storage/src/batch/commit.rs`, `JournalWriteBatch::strict` and `commit` (observed lines 5–40). `strict` selects Fjall `PersistMode::SyncAll`; aborted batches return a typed error before commit. Related test: `src/tests/batch_write_through.rs`, `journal_close_and_reopen_preserves_strict_data` (observed lines 654–678). It checks the complete recovered header after reopening; this is not a power-loss test or proof of every commit path.

Keep only the pair relevant to the task in a worker packet. Re-check symbols/line ranges and dependency versions rather than copying this snapshot indefinitely.
