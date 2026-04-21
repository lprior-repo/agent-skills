# `bd` Workflow: The Autonomous Execution Doctrine

## 1. Synchronize (The Handshake)
Before any action, ensure your internal context matches the project's external memory.
- `bd sync`: Pulls latest issues from `issues.jsonl` into SQLite.
- `bd doctor`: Verifies integrity. If issues are found, use `--fix`.

## 2. Triage (The Strategic Choice)
Identify the "Execution Frontier" using the dependency graph.
- `bd ready --json`: Fetches all tasks with ZERO open blockers.
- `bv --robot-insights`: Use for complex prioritization:
    - **Foundational Work**: High PageRank (unblocks the future).
    - **Risk Reduction**: High Betweenness Centrality (bridges silos).
    - **Value Delivery**: High Hub scores (Epics aggregating value).

## 3. Claim & Execute
Lock the task to prevent coordination conflicts in a swarm.
- `bd update <ID> --status in_progress --assignee self`
- `bd show <ID> --json`: Read description, parent epics, and comments.

## 4. Organic Discovery (Genealogy)
If you find a bug or missing requirement during execution, do NOT hold it in your context window.
- `bd create "Title" --deps discovered-from:<Current-ID>`
- `bd dep add <New-ID> <Current-ID>`: If it blocks you.

## 5. Finalize (The Landing)
- `bd update <ID> --status closed`: Mark as done.
- `git commit -m "Fix logic (ref bd-a1b2)"`: Reference the ID.
- `bd sync && git push`: Non-negotiable. The session is not complete until this succeeds.

### Conflict Resolution (3-Way Merge)
- **Deletions Win**: If one agent modifies and another closes, the task is closed.
- **Union of Additions**: Both new tasks are kept.
- **Protocol**: `git pull --rebase` -> `bd sync` -> `git push`.
