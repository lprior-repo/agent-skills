# Refinery Cron Protocol

## Cron Schedule
Every 10 minutes.

## Cron Prompt
```
REFINERY CRON: For each rig, discover unmerged branches, classify, merge smallest first (cherry-pick for deep branches), build-check, push, prune. Report merged/skipped/pruned counts. Skip rigs with no unmerged branches.
```

## Report Format

```
REFINERY REPORT — <timestamp>
┌─────────────┬────────────┬────────┬─────────┬────────┬────────────┬────────┐
│ Rig         │ Discovered │ Merged │ Skipped │ Pruned │ Remaining  │ Build  │
├─────────────┼────────────┼────────┼─────────┼────────┼────────────┼────────┤
│ veloxide    │ 39         │ 2      │ 5       │ 3      │ 29         │ PASS   │
│ hardline    │ 0          │ 0      │ 0       │ 0      │ 0          │ PASS   │
│ oya_frontend│ 8          │ 1      │ 2       │ 1      │ 4          │ PASS   │
└─────────────┴────────────┴────────┴─────────┴────────┴────────────┴────────┘
```

## Escalation Rules
- If 3+ branches skip in a row with conflicts → nudge user
- If build fails after merge → revert and log
- If git push fails → investigate, do NOT retry blindly
