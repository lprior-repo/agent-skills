---
name: disk-cleanup
description: Audit disk use and perform bounded daily cleanup on Lewis's Omarchy Linux system. Use for disk pressure, cleanup candidates, Rust build targets, or stale cache directories; preserve personal files, Steam, LLM models, and live data stores.
metadata:
  short-description: Bounded daily disk audit and cleanup
---

# Disk cleanup

Use this skill for a careful, repeatable disk audit and cleanup on this system. The normal cleanup age is **1 day**. A user systemd timer runs the guarded cleanup daily; the skill also supports manual audits and cleanup.

## Keep rules

- Never modify `~/Documents` or `~/Downloads`. They are personal files. Report directories and individual files over 3 GiB only.
- Preserve Steam games and all configured LLM/model stores. Check configured model paths and running model services rather than guessing from directory names.
- Preserve source files, worktrees, project `var/` data, databases, backups, snapshots, receipts, evidence, and fuzz corpora unless the user identifies a specific item for cleanup and its ownership and references have been checked.
- Never stop model or data services, delete a live database, or discard paused invocation history as part of routine cleanup.
- Do not empty Trash unless the user explicitly asks. Moving files there does not reclaim disk space until Trash is emptied.

## Daily workflow

1. Check actual filesystem use with `df -hT`. On Btrfs, also inspect `btrfs filesystem usage -T /`. Logical directory totals from `dust` can count shared extents more than once; use filesystem-level numbers for actual used/free space.
2. Inventory only relevant paths with bounded `dust -d 1 <path>` snapshots. Avoid a full recursive `du` over the home directory. Keep Documents and Downloads read-only; list large directories and files over 3 GiB for the user.
3. For routine cleanup, preview the Rust CLI first:

   ```sh
   target-reaper --root "$HOME" --cache-root "$HOME/.cache" --cache-root /cache \
     --worktree-repo "$HOME/src/ad-law-scrape/athletic-rust-pipeline" \
     --age-days 1 --min-size 10MB --list
   ```

   The CLI searches `~/` for eligible Cargo `target` directories, inspects only immediate cache-root children, and checks linked worktrees registered to the specified repository. Documents, Downloads, Steam, model stores, local databases, containers, and other protected paths are excluded. It keeps fresh or active targets, caches in use, and worktrees that are unmerged, dirty, locked, active, or otherwise protected. Review the preview before execution.

4. When the user asks to run the established `rust-clean` cleanup, run the same CLI command with `--execute --list`, keeping `--age-days 1`. The `rust-clean` shell alias calls the CLI in execution mode; it is not a preview command. Do not broaden its roots, lower its age threshold, or add other deletion categories unless asked.
5. For anything outside those generated targets and stale cache directories, inspect the contents, owning process, project references, and relevant runbooks first. For Fjall/Restate/census stores, follow the repository's lifecycle and cold-backup procedures. If more than one active store/run exists, report the evidence and ask which one to retain before deleting durable data.
6. Verify with `df -hT` after cleanup. Report exactly which paths were removed, the helper's reported reclaimed amount, and the before/after filesystem usage. Reclamation can lag while open files are held by processes.

## Commands and cautions

- CLI: `~/.local/bin/target-reaper`; the Rust CLI is canonical for targets, caches, and configured worktrees. The old `scripts/rust_clean.py` helper is not used by the alias or daily timer.
- Use `dust` for concise directory inventories and `df`/Btrfs usage for physical capacity.
- Do not use broad `rm -rf`, `sudo` cleanup, package-manager purges, or process termination for routine cleanup.
- A cache name alone does not prove that its contents are disposable. Honor the helper's protected paths and names, and keep ambiguous or actively used data for review.
