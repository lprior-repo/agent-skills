# QA Enforcer Implementation Guide

## JSONL Structure Reference

The SKILL.md contains a JSONL-encoded block (lines 24-86) that mechanically encodes:
- **Principles**: Core testing philosophy (execute_everything, evidence_required, deep_inspection, fix_or_report)
- **Test Rules**: Mandatory behaviors (actual_execution, real_output_only, comprehensive_coverage)
- **Categories**: Test domains (cli, api, adversarial, workflow)
- **Checks**: Specific validations with severity levels (exit_codes, error_messages, panic_detection, secret_leak, etc.)
- **Patterns**: Reusable test templates (smoke_test, integration_test, regression_test, chaos_file_operations)
- **Severity**: Issue classification (critical, major, minor, observation)
- **Auto-fixes**: Automatically fixable patterns with detection signatures
- **Anti-patterns**: What to avoid with rationale
- **Gates**: Quality gates that must pass before sign-off
- **Commands**: CLI interface definitions
- **Examples**: Code examples for each category
- **Integrations**: How this skill integrates with red-queen, tcr-enforcer, zjj

Agents can parse this JSONL to:
- Look up severity by criteria
- Find auto-fix patterns by detection signature
- Extract examples by category
- Validate compliance against encoded rules
- Reference quality gates for sign-off decisions

## How This Skill Works in Practice

When a user invokes `qa-enforcer`, the agent follows this exact workflow:

### Entry Point

User says:
- "Test this CLI thoroughly"
- "QA this API endpoint"
- "Break this for me"
- "Does this actually work?"
- "qa-enforcer test <target>"

### Phase 0: Target Discovery (5-10 seconds)

```bash
# 1. Identify what we're testing
if argument is a file:
  target_type = "file"
  file_path = argument
elif argument is a directory:
  target_type = "project"
  project_path = argument
elif argument is a URL:
  target_type = "api"
  base_url = argument
elif argument is empty:
  target_type = "auto-detect"
  scan cwd for targets

# 2. Gather context
if target_type in ["file", "project"]:
  read README.md if exists
  read package.json/Cargo.toml/gleam.toml
  find executables
  find test files
elif target_type == "api":
  try /swagger.json or /openapi.json
  try /api/docs
  grep router definitions in code
```

### Phase 1: Test Planning (1-2 minutes)

```bash
# Create comprehensive test plan
echo "Test Plan for: $target"
echo "="
echo "Target Type: $target_type"

# CLI tests
if target is CLI:
  discover_subcommands
  discover_flags
  discover_examples_from_docs

# API tests
if target is API:
  discover_endpoints
  discover_methods
  discover_auth_requirements

# Product owner tests
read_user_stories_from_issues
read_acceptance_criteria
```

### Phase 2: Test Execution (5-15 minutes)

```bash
# Execute EVERY test, capture EVERYTHING
run_test_suite() {
  local test_name="$1"
  local command="$2"
  local expected_exit="${3:-0}"

  echo "▶ $test_name"
  echo "Command: $command"

  # Create temp file for output capture
  local tmpout=$(mktemp)
  local tmperr=$(mktemp)

  # Run with precise measurement
  /usr/bin/time -v bash -c "$command" >"$tmpout" 2>"$tmperr"
  local exit_code=$?

  # Read outputs
  local stdout=$(cat "$tmpout")
  local stderr=$(cat "$tmperr")

  # Print results
  echo "Exit Code: $exit_code (expected: $expected_exit)"
  echo "Stdout: $stdout"
  echo "Stderr: $stderr"

  # Clean up
  rm -f "$tmpout" "$tmperr"

  # Return result
  if [[ $exit_code -eq $expected_exit ]]; then
    echo "✅ PASS"
    return 0
  else
    echo "❌ FAIL"
    return 1
  fi
}
```

### Phase 3: Deep Inspection (2-5 minutes per test)

```bash
# After each test, analyze output DEEPLY
deep_inspect() {
  local output="$1"
  local test_context="$2"

  local issues=()

  # 1. Panic detection (ZERO TOLERANCE)
  if grep -qiE "panic|todo!|unimplemented!|unwrap.*failed" <<< "$output"; then
    issues+=("CRITICAL: Panic/todo in output - never acceptable in user-facing code")
  fi

  # 2. Error message quality
  if grep -qiE "error|failed" <<< "$output"; then
    local error_msg=$(grep -iE "error:|failed:" <<< "$output" | head -1)
    if [[ -n "$error_msg" ]]; then
      # Check if error has actionable information
      if grep -qE "(at .+:[0-9]+|caused by:|suggestion:)" <<< "$error_msg"; then
        echo "✅ Error message has context"
      else
        issues+=("MAJOR: Error message lacks context - not actionable")
      fi
    fi
  fi

  # 3. Security scan
  if grep -iE "(password|token|secret|api_key|private_key)=" <<< "$output"; then
    issues+=("CRITICAL: Possible secret leaked in output")
  fi
  if grep -iE "stack trace|backtrace" <<< "$output"; then
    issues+=("MAJOR: Stack trace in user output - implementation detail leak")
  fi

  # 4. Performance indicators
  if grep -qE "took.*ms|elapsed|duration" <<< "$output"; then
    local duration=$(echo "$output" | grep -oE "[0-9]+(\.[0-9]+)?(ms|s)" | head -1)
    echo "ℹ️  Duration: $duration"
  fi

  # 5. Output format consistency
  local line_count=$(echo "$output" | wc -l)
  if [[ $line_count -gt 100 ]]; then
    issues+=("MINOR: Very long output ($line_count lines) - consider summarization")
  fi

  # 6. Empty output check
  if [[ -z "$output" ]]; then
    issues+=("OBSERVATION: No output produced - is this expected?")
  fi

  # 7. Whitespace issues
  if grep -q " $" <<< "$output"; then
    issues+=("MINOR: Trailing whitespace in output")
  fi

  # Return all issues
  printf '%s\n' "${issues[@]}"
}
```

### Phase 4: Issue Handling (immediate per finding)

```bash
# For each issue found, decide action
handle_finding() {
  local severity="$1"
  local description="$2"
  local command="$3"
  local output="$4"

  echo "═════════════════════════════════════"
  echo "ISSUE FOUND: $severity"
  echo "═════════════════════════════════════"
  echo "Description: $description"
  echo "Reproduction: $command"
  echo "Output:"
  echo "$output"

  # Determine if auto-fixable
  if is_auto_fixable "$description" "$output"; then
    echo "Attempting auto-fix..."
    if attempt_fix "$severity" "$description" "$command"; then
      echo "✅ Auto-fixed successfully"
      # Re-test to verify
      run_test_suite "Verify fix" "$command"
      return $?
    fi
  fi

  # If we can't fix, file an issue
  echo "Filing issue to tracker..."
  file_bead "$severity" "$description" "$command" "$output"
}

# Auto-fix detection
is_auto_fixable() {
  local description="$1"
  local output="$2"

  # Check for known auto-fixable patterns
  if grep -qi "permission denied" <<< "$output"; then
    return 0  # Can chmod
  elif grep -qi "no such file" <<< "$output"; then
    return 0  # Can create file
  elif grep -qi "trailing whitespace" <<< "$description"; then
    return 0  # Can trim
  elif grep -qi "missing newline" <<< "$description"; then
    return 0  # Can add
  fi

  return 1  # Not auto-fixable
}

# Attempt fix
attempt_fix() {
  local severity="$1"
  local description="$2"
  local command="$3"

  case "$description" in
    *"permission denied"*)
      # Extract file path and chmod
      local file=$(echo "$command" | grep -oE '[^ ]+\.[a-z]+$' | head -1)
      if [[ -n "$file" ]]; then
        chmod +x "$file"
        echo "Made executable: $file"
        return 0
      fi
      ;;

    *"trailing whitespace"*)
      # Find and fix
      local file=$(echo "$command" | grep -oE '[^ ]+\.[a-z]+$' | head -1)
      if [[ -n "$file" ]]; then
        sed -i 's/[[:space:]]*$//' "$file"
        echo "Trimmed trailing whitespace from: $file"
        return 0
      fi
      ;;

    *"missing newline"*)
      local file=$(echo "$command" | grep -oE '[^ ]+\.[a-z]+$' | head -1)
      if [[ -n "$file" ]]; then
        # Check if file ends with newline
        if [[ $(tail -c1 "$file" | wc -l) -eq 0 ]]; then
          echo >> "$file"
          echo "Added newline to: $file"
          return 0
        fi
      fi
      ;;
  esac

  return 1  # Fix failed
}
```

### Phase 5: Reporting (comprehensive summary)

```bash
# After all tests, generate comprehensive report
generate_report() {
  local total_tests=$1
  local passed_tests=$2
  local issues_found=$3
  local issues_fixed=$4

  cat <<EOF
QA ENFORCER REPORT
═══════════════════════════════════════════════════════════════

Target:       $TARGET
Date:         $(date -Iseconds)
Tests Run:    $total_tests
Passed:       $passed_tests
Failed:       $((total_tests - passed_tests))
Issues Found: $issues_found
Issues Fixed: $issues_fixed

EXECUTION SUMMARY
═══════════════════════════════════════════════════════════════

$passed_tests/$total_tests tests passed ($(( passed_tests * 100 / total_tests ))%)

ISSUES BY SEVERITY
═══════════════════════════════════════════════════════════════

CRITICAL: $(count_issues CRITICAL)
MAJOR:    $(count_issues MAJOR)
MINOR:    $(count_issues MINOR)

DETAILED FINDINGS
═══════════════════════════════════════════════════════════════

$(list_findings)

PRODUCT OWNER VALIDATION
═══════════════════════════════════════════════════════════════

$(validate_user_workflow)

RECOMMENDATIONS
═══════════════════════════════════════════════════════════════

$(generate_recommendations)

═══════════════════════════════════════════════════════════════
Overall Verdict: $(determine_verdict)
EOF
}

# Determine verdict
determine_verdict() {
  local critical_count=$(count_issues CRITICAL)

  if [[ $critical_count -gt 0 ]]; then
    echo "❌ DON'T SHIP - Critical issues must be fixed"
  elif [[ $passed_tests -lt $((total_tests * 80 / 100)) ]]; then
    echo "⚠️  FAIL - Less than 80% tests passing"
  else
    echo "✅ PASS - Ready to ship"
  fi
}
```

## Example Session: Testing a CLI

```bash
# User invocation
User: "qa-enforcer test oya"

# Phase 0: Discovery
QA: Found CLI: /home/lewis/.local/bin/oya
    Type: Rust binary
    README: /home/lewis/src/oya/README.md

# Phase 1: Planning
QA: Test Plan:
    ├─ CLI Invocation Tests (8 tests)
    ├─ Output Quality Tests (5 tests)
    ├─ Error Handling Tests (6 tests)
    └─ User Workflow Tests (3 tests)
    Total: 22 tests

# Phase 2: Execution
QA: ▶ Testing: Binary exists
    Command: which oya
    Exit Code: 0 (expected: 0)
    Stdout: /home/lewis/.local/bin/oya
    ✅ PASS

    ▶ Testing: Help flag works
    Command: oya --help
    Exit Code: 0 (expected: 0)
    Stdout: [full help output]
    ✅ PASS

    ▶ Testing: Invalid command fails
    Command: oya nonexistent-command
    Exit Code: 1 (expected: 1)
    Stderr: error: unrecognized subcommand 'nonexistent-command'
    ✅ PASS

    ▶ Testing: New command without slug
    Command: oya new
    Exit Code: 2 (expected: 1)
    Stderr: error: required argument '-s <slug>' not found
    ⚠️  UNEXPECTED EXIT CODE
    Analysis: Expected exit 1, got 2. Error message is clear though.
    Severity: MINOR
    ❌ FAIL

    ... (continues for all 22 tests)

# Phase 3: Deep Inspection
QA: Analyzing "new command without slug" failure...
    Issue: Exit code mismatch
    Severity: MINOR
    Expected: 1 (generic error)
    Actual: 2 (argument parsing error)
    Recommendation: Document expected exit codes, or align with conventions
    Action: OBSERVATION logged

# Phase 4: Issue Handling
QA: Found 3 issues:
    1. MINOR: Exit code mismatch - Not auto-fixable
       → Filed bead: #1234

    2. MAJOR: Error message lacks context - Not auto-fixable
       → Filed bead: #1235

    3. CRITICAL: Panic in error path - Auto-fixable
       → Attempting fix...
       ✅ Fixed: Added proper error handling
       → Re-testing... ✅ PASS

# Phase 5: Report
QA ENFORCER REPORT
═══════════════════════════════════════════════════════════════

Target:       oya
Tests Run:    22
Passed:       20
Failed:       2
Issues Found: 3
Issues Fixed: 1

EXECUTION SUMMARY
═══════════════════════════════════════════════════════════════

20/22 tests passed (91%)

ISSUES BY SEVERITY
═══════════════════════════════════════════════════════════════

CRITICAL: 0 (1 fixed)
MAJOR:    1
MINOR:    1

RECOMMENDATIONS
═══════════════════════════════════════════════════════════════

1. Document expected exit codes in README
2. Add context to "required argument not found" error message
   - Suggestion: "The 'new' command requires a task slug. Example: oya new -s my-task"

═══════════════════════════════════════════════════════════════
Overall Verdict: ✅ PASS - Ready to ship (minor issues filed)
```

## Example Session: API Testing

```bash
# User invocation
User: "qa-enforcer api http://localhost:8080"

# Phase 0: Discovery
QA: Testing API: http://localhost:8080
    Found OpenAPI spec: /swagger.json
    Endpoints discovered: 12

# Phase 1: Planning
QA: Test Plan:
    ├─ GET endpoints (8 tests)
    ├─ POST endpoints (3 tests)
    ├─ DELETE endpoints (1 test)
    ├─ Authentication tests (4 tests)
    ├─ Input validation tests (6 tests)
    └─ Security tests (5 tests)
    Total: 27 tests

# Phase 2: Execution
QA: ▶ Testing: GET /api/tasks
    Command: curl -s http://localhost:8080/api/tasks
    Status: 200
    Body: [{"id":"task1","status":"pending"}]
    Response Time: 45ms
    ✅ PASS

    ▶ Testing: POST /api/tasks without auth
    Command: curl -s -X POST http://localhost:8080/api/tasks -d '{"title":"test"}'
    Status: 401
    Body: {"error":"Unauthorized"}
    ✅ PASS

    ▶ Testing: GET /api/tasks with invalid ID
    Command: curl -s http://localhost:8080/api/tasks/nonexistent-id
    Status: 500
    Body: Internal Server Error
    ❌ FAIL
    Analysis: Should return 404, not 500
    Severity: MAJOR
    Issue filed: #1236

    ... (continues)

# Deep Inspection on failing test
QA: Analyzing "GET with invalid ID returns 500"...
    Expected: 404 Not Found
    Actual: 500 Internal Server Error
    Root cause: Unhandled .expect() in database query
    Suggested fix:
      ```rust
      // Before:
      let task = db.get_task(id).expect("Task not found");

      // After:
      let task = db.get_task(id)?;
      ```
    Action: Filed bead #1236 with full details

# Final Report
QA ENFORCER REPORT
═══════════════════════════════════════════════════════════════

API:          http://localhost:8080
Tests Run:    27
Passed:       24
Failed:       3
Issues Found: 4
Issues Fixed: 0

CRITICAL ISSUES
═══════════════════════════════════════════════════════════════

❌ SQL Injection vulnerability in /api/tasks search
   → Filed bead: #1237
   → Blocked: YES

MAJOR ISSUES
═══════════════════════════════════════════════════════════════

⚠️  Invalid ID returns 500 instead of 404
   → Filed bead: #1236

⚠️  Missing rate limiting on public endpoints
   → Filed bead: #1238

═══════════════════════════════════════════════════════════════
Overall Verdict: ❌ DON'T SHIP - Critical security issue
```

## Example Session: User Workflow Testing

```bash
# User invocation
User: "qa-enforcer workflow 'Create task and approve it'"

# Phase 0: Discovery
QA: Testing workflow: Create task and approve it
    Reading user stories...

# Phase 1: Planning
QA: Workflow steps:
    1. oya new -s test-task
    2. Verify task created (br show test-task)
    3. oya approve -s test-task
    4. Verify approved status

# Phase 2: Execution
QA: ▶ Step 1: Create task
    Command: oya new -s qa-workflow-test
    Exit Code: 0
    Output: Created task: qa-workflow-test
    ✅ PASS

    ▶ Step 2: Verify task exists
    Command: br show qa-workflow-test
    Exit Code: 0
    Output:
      ID: qa-workflow-test
      Status: pending
      Created: 2026-02-06
    ✅ PASS

    ▶ Step 3: Approve task
    Command: oya approve -s qa-workflow-test
    Exit Code: 1
    Output: error: Task 'qa-workflow-test' is not in 'approvable' state
    ❌ FAIL

    Analysis: Workflow broken!
    User expectation: Can approve any pending task
    Actual behavior: Must move through states first
    Severity: MAJOR (UX issue - doesn't match user story)

    Deep inspection:
    - Documentation says "Approve task when ready"
    - But code requires transition to 'review' state first
    - Missing step: oya stage -s qa-workflow-test --stage review
    - Either: Update docs or fix workflow

    Recommendation: Update command help to show workflow:
      "oya new → oya stage → oya approve"

# Phase 3: Issue Handling
QA: Filing MAJOR issue: Workflow doesn't match user expectation
    → Bead #1239 created
    → Suggestion: Add 'oya workflow' command to show state diagram

# Final Report
QA ENFORCER REPORT
═══════════════════════════════════════════════════════════════

Workflow:     Create task and approve it
Steps Passed: 2/4
Status:       ❌ FAIL

PRODUCT OWNER VALIDATION
═══════════════════════════════════════════════════════════════

User Goal: Create and approve a task
Result:     INCOMPLETE
Blocker:    State transition not documented

UX Quality: ⚠️  CONFUSING
├─ Error message is technically correct but unhelpful
├─ No indication of what state is required
└─ User must read source code to understand workflow

Recommendation:
1. Add state diagram to README
2. Update error message: "Task must be in 'review' state first. Run: oya stage -s <task> --stage review"
3. Consider: Add 'oya workflow' command to show current state and available transitions

═══════════════════════════════════════════════════════════════
Overall Verdict: ❌ FAIL - UX doesn't meet user needs
```

## Integration with Other Skills

### After QA Enforcer Finds Issues

```bash
# Option 1: Fix manually with zjj isolation
zjj add qa-fix-$(date +%s)
# Make fixes
zjj done  # Merges when tests pass

# Option 2: Feed into red-queen for regression prevention
red-queen add-done-when-from-qa \
  --command "oya new" \
  --expect-exit 0 \
  --dimension "cli-creation"

# Option 3: Use tcr-enforcer to prevent regressions
tcr-enforcer start qa-fix-branch
# Make changes, TCR auto-commits if tests pass
```

## Tips for Maximum Effectiveness

1. **Always capture the full output** - Don't summarize, show everything
2. **Never assume** - If you didn't run it, you don't know it works
3. **Test like a user** - Follow documentation, not code structure
4. **Be ruthless** - Find every edge case, break everything
5. **Provide evidence** - Every issue must be reproducible
6. **Fix what you can** - Don't just report, act
7. **Think like a product owner** - Does this solve the actual problem?
8. **Document expectations** - What should happen vs what actually happens

## Common Mistakes to Avoid

❌ "The code looks correct" → DID YOU RUN IT?
❌ "Tests should pass" → DID YOU RUN THE TESTS?
❌ "No obvious issues" → DID YOU TRY TO BREAK IT?
❌ "Happy path works" → DID YOU TEST ERROR CASES?
❌ "Error handling exists" → DID YOU TRIGGER ERRORS?

**QA Enforcer never assumes. QA Enforcer executes.**
