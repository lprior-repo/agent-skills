---
name: refinery
description: "Merge unmerged polecat branches into main across all Gas Town rigs. Cherry-pick or merge, resolve conflicts, prune dead branches, keep main clean. Invoke on cron or when user asks to clean up branches."
allowed-tools:
  - Read
  - Bash
  - Grep
  - Glob
---

```jsonl
{"kind":"meta","skill":"refinery","version":"1.0.0","format":"jsonl-progressive","mode":"contract-first","token_strategy":"top_block_plus_jsonl"}
{"kind":"input","arguments":"$ARGUMENTS","rule":"First arg is rig name or 'all'. Optional --dry-run shows plan without executing. Optional --force attempts conflict resolution on uncertain branches."}
{"kind":"mission","goal":"Keep main branches clean across all Gas Town rigs by merging unmerged polecat branches, resolving conflicts safely, and pruning dead branches. Trunk-based development: main is always deployable."}
{"kind":"rigs","list":[{"name":"veloxide","dir":"/home/lewis/src/veloxide","build":"moon run :ci","quick":"moon run :quick","prefix":"ve-"},{"name":"hardline","dir":"/home/lewis/src/hardline","build":"cargo test","quick":"cargo check","prefix":"ha-"},{"name":"oya_frontend","dir":"/home/lewis/src/oya-frontend","build":"cargo test","quick":"cargo check","prefix":"of-"},{"name":"twerk","dir":"/home/lewis/src/twerk","build":"cargo test","quick":"cargo check","prefix":"tw-"},{"name":"Seshat","dir":"/home/lewis/src/Seshat","build":"cargo test","quick":"cargo check","prefix":"se-"}]}
{"kind":"rule","id":"never_lose_work","text":"NEVER git push --force to main (only --force-with-lease). NEVER delete a branch without verifying its commits are reachable from main. NEVER discard uncommitted changes. NEVER operate on dirty working tree. If uncertain, SKIP and report."}
{"kind":"rule","id":"preflight_clean_tree","text":"BEFORE any merge attempt, run git diff --quiet && git diff --cached --quiet. If EITHER fails, ABORT — dirty tree means potential data loss on reset."}
{"kind":"rule","id":"full_verify","text":"Use rig 'build' command (moon run :ci or cargo test) for final verification after ALL merges complete. Use 'quick' command (moon run :quick or cargo check) after each individual merge for fast feedback."}
{"kind":"rule","id":"cherry_pick_deep","text":"When branch is >50 commits behind main, use cherry-pick not merge. Deep merge causes cascading conflicts."}
{"kind":"rule","id":"one_at_a_time","text":"Merge ONE branch at a time. Verify build after each. Revert immediately on failure."}
{"kind":"rule","id":"conflict_policy","text":"Conflicts: formatting/style -> keep HEAD. New logic -> keep incoming. Uncertain -> SKIP branch."}
{"kind":"rule","id":"prune_after_merge","text":"After successful merge + build pass + push: delete remote branch. Then git fetch --prune."}
{"kind":"rule","id":"build_gate","text":"After every merge, run rig build check. Failure = git reset --hard origin/main, skip branch."}
{"kind":"rule","id":"anti_hallucination","text":"NEVER fabricate git output, branch names, or conflict resolutions. Run actual commands."}
{"kind":"workflow","id":"refinery_cycle","steps":["Discover: git fetch --prune, list unmerged branches","Classify: compute ahead/behind for each branch","Sort: smallest ahead count first","Merge: one branch at a time with strategy selection","Verify: build check after each merge","Push: only after build passes","Prune: delete merged remote branch","Report: merged/skipped/pruned counts per rig"]}
{"kind":"strategy","id":"direct_merge","when":"behind <= 50 AND ahead <= 10","cmd":"git merge origin/$branch --no-edit","risk":"low"}
{"kind":"strategy","id":"cherry_pick","when":"behind > 50 OR ahead > 10","cmd":"git log --reverse --format=%H origin/main..origin/$branch then git cherry-pick each","risk":"medium"}
{"kind":"strategy","id":"stale_prune","when":"ahead == 0","cmd":"git push origin --delete $branch","risk":"none"}
{"kind":"conflict_resolution","id":"formatting","detect":"diff is whitespace/indentation only","action":"git checkout --ours"}
{"kind":"conflict_resolution","id":"new_logic","detect":"incoming adds new functions/types/imports","action":"git checkout --theirs -- <file> then git add"}
{"kind":"conflict_resolution","id":"uncertain","detect":"both sides have meaningful changes","action":"SKIP branch, report for manual review"}
{"kind":"gate","id":"build_check","text":"Run rig build command after EVERY merge. FAIL = revert immediately."}
{"kind":"gate","id":"push_verify","text":"After git push, run git status to confirm up to date with origin."}
{"kind":"gate","id":"branch_reachability","text":"Before deleting remote branch, git log origin/main..origin/$branch must show empty output."}
{"kind":"output_contract","for":"refinery_report","format":"table","columns":["rig","discovered","merged","skipped","pruned","remaining","build"]}
{"kind":"cron","schedule":"every 10 minutes","prompt":"REFINERY CRON: For each rig, discover unmerged branches, classify, merge smallest first, build-check, push, prune. Report merged/skipped/pruned."}
{"kind":"ref","file":"references/strategies.md","use":"Merge strategy decision tree and conflict resolution patterns"}
{"kind":"ref","file":"references/cron-protocol.md","use":"Cron trigger format and reporting template"}
{"kind":"ref","file":"references/safety-checklist.md","use":"Pre-flight safety checks before destructive git operations"}
```

Progressive disclosure:
- [references/strategies.md](references/strategies.md) — Merge decision tree, cherry-pick patterns, conflict examples
- [references/cron-protocol.md](references/cron-protocol.md) — Cron trigger format and reporting
- [references/safety-checklist.md](references/safety-checklist.md) — Pre-flight checks and revert procedures

## Mandatory Verification Gate

After ANY refinery operation, you MUST run these commands and observe PASS:

```bash
# 1. Working tree clean
cd <rig-source-dir>
git diff --quiet && echo "PASS: clean tree" || echo "FAIL: dirty tree"

# 2. On main and up to date
git branch --show-current | grep -q main && echo "PASS: on main" || echo "FAIL: not on main"
git status | grep -q "up to date" && echo "PASS: up to date" || echo "FAIL: not pushed"

# 3. Build + lint + tests pass (full verification)
# Veloxide: moon run :ci
# Others: cargo test
<rig-build-command>

# 4. Count remaining unmerged
git branch -r --no-merged origin/main | grep -v "HEAD\|main$" | wc -l
```
