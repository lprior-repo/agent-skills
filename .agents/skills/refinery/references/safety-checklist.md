# Refinery Safety Checklist

## Pre-Flight (before ANY merge)
- [ ] `git status` shows clean working tree
- [ ] `git fetch --prune origin` succeeds
- [ ] Current branch is main
- [ ] main is up to date with origin/main

## During Merge
- [ ] Only ONE branch being processed at a time
- [ ] Strategy matches classification (merge vs cherry-pick)
- [ ] Conflicts handled per conflict resolution rules
- [ ] No force-push to main

## Post-Merge (per branch)
- [ ] Build check passes
- [ ] `git push origin main` succeeds
- [ ] `git status` shows up to date
- [ ] Remote branch deleted (only after push succeeds)

## Post-Flight (after full run)
- [ ] All rigs have clean main branches
- [ ] No stale remote refs (git fetch --prune ran)
- [ ] Report generated with accurate counts
- [ ] Any skipped branches documented with reason

## Emergency Recovery
```bash
# If stuck mid-merge:
git merge --abort

# If stuck mid-cherry-pick:
git cherry-pick --abort

# If main is broken after push:
git reset --hard origin/main~1  # Local revert
git push origin main --force-with-lease  # Push revert
```

## NEVER
- NEVER `git push --force` (only `--force-with-lease` is allowed)
- NEVER skip build check or test suite
- NEVER batch-merge without verification
- NEVER delete a branch before confirming commits are in main
- NEVER operate on a dirty working tree (git diff --quiet MUST pass first)
- NEVER cherry-pick more than one commit without checking each individually
