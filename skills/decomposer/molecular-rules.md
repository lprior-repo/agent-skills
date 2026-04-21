# Molecular Slicing Rules

To guarantee that a task has been shredded into a truly "molecular" size, the Decomposer relies on 6 objective heuristics. 

### 1. The Boundary Isolation Rule (Data → Calc → Actions)
A task is too big if it crosses architectural boundaries. It must live in exactly **one** layer.
*   **Fail:** "Update database schema, update API payload, and show error in UI." (Touches Data, Calc, and Action).
*   **Pass:** "Add UNIQUE constraint to `events` table and write raw SQL test." (Data only).
*   **Pass:** "Update Dioxus reducer to handle `RevisionConflict` state." (Action only).

### 2. The PR Diff Limit (Blast Radius)
A task is molecular if the resulting code change is expected to be **under 100 lines of logic** (excluding tests/boilerplate) and touches **no more than 3 existing files**.
*   **Why:** If a change touches 15 files, it is an Epic disguised as a task. AI hallucination risk scales exponentially with the number of files modified.

### 3. The Revertability Rule (Safe Deployability)
Can this task be merged into `main` and deployed to production **right now**, even if the rest of the feature isn't finished? 
*   **Why:** Molecular tasks must be "dormant" or backward-compatible. If merging Task 1 breaks the app until Task 2 and 3 are finished, the task was sliced incorrectly. 

### 4. The Isolated Verifiability Rule (No "God Mocks")
Can you prove this task works without spinning up the entire stack?
*   **Fail:** "Test requires firing up the Dioxus UI, the Restate cluster, and the SQLite DB."
*   **Pass:** "Can be proven by a single, pure Rust unit test passing in `< 2 seconds`." 
*   If testing the task requires massive orchestration or complex mocks, the task is too large.

### 5. The "Single Failure Mode" Rule
Can you list all the ways this specific code change could fail? If there are more than 2 or 3 distinct error variants introduced by this task, it's doing too much.

### 6. The "Time to Green" Rule
If an AI agent were to implement this task, would it hit a passing test suite within **15 minutes** of starting? If the feedback loop requires an hour of trial and error, the contract is too broad.
