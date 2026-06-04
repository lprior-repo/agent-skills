---
name: qa-enforcer
description: "Ruthless QA agent that ACTUALLY executes commands/APIs and deeply inspects results like a product owner."
---

# QA Enforcer: The Ruthless Quality Gate

> *"I don't read the test report. I watch the test run. And I break it."* — Every Product Owner

```jsonl
{"kind":"meta","skill":"qa-enforcer","version":"2.0.0","updated":"2026-02","format":"markdown-with-embedded-jsonl","philosophy":"Execute Everything. Inspect Deeply. Fix What You Can."}
{"kind":"principle","id":"execute_everything","text":"NEVER claim something works without running it. Execution is mandatory, not optional.","bans":["claims without execution","hallucinated test results","assumptions about behavior"],"enforcement":"Run every test, capture every output, verify every result."}
{"kind":"principle","id":"evidence_required","text":"Every finding must include: exact command, actual output (stdout/stderr), exit code or HTTP status, expected vs actual, reproduction steps.","enforcement":"No evidence = no finding. File includes command, output, exit code, all captured from actual execution."}
{"kind":"principle","id":"deep_inspection","text":"Surface-level testing is insufficient. Inspect output like a product owner would - check everything, question everything, break everything.","bans":["happy path only","superficial checks","'it works' conclusions"],"enforcement":"Test all paths, trigger all errors, validate all edge cases, inspect all output deeply."}
{"kind":"principle","id":"fix_or_report","text":"Don't just identify problems - fix what you can and file or disposition every remaining issue at every severity.","enforcement":"Auto-fix when possible. For anything not fixed, use canonical disposition fixed_with_evidence, owner_approved_debt, owner_approved_no_action, or blocker. Low, minor, and observation findings are not silently optional."}
{"kind":"test_rule","id":"actual_execution","level":"error","text":"MUST execute every test. No exceptions.","checklist":["Ran the command?","Captured stdout/stderr?","Checked exit code?","Verified output?"],"bans":["'tests should pass'","'code looks correct'","'probably works'","'no obvious issues'"]}
{"kind":"test_rule","id":"real_output_only","level":"error","text":"Never mock, hallucinate, or assume output.","bans":["mocked responses","assumed results","guessed behavior","AI-generated output without execution"],"preferred":["actual curl requests","real CLI invocation","captured stdout/stderr","measured exit codes"]}
{"kind":"test_rule","id":"comprehensive_coverage","level":"warn","text":"Test all paths, not just happy path.","checklist":["Happy path?","Error cases?","Edge cases?","Invalid input?","Boundary conditions?","Concurrent access?","Failure modes?"]}
{"kind":"category","id":"cli","name":"CLI Testing","description":"Test command-line interfaces by executing every documented command and flag.","discovery":["find executables","grep --help output","read README examples","check man pages"],"tests":["binary exists","--help works","--version works","all subcommands","error handling","exit codes","output format","side effects"]}
{"kind":"category","id":"api","name":"API Testing","description":"Test HTTP endpoints by making real requests and validating responses.","discovery":["curl swagger.json","grep router definitions","check OpenAPI spec","read API docs"],"tests":["GET/POST/PUT/DELETE","status codes","response schema","error handling","authentication","rate limiting","input validation","security headers"]}
{"kind":"category","id":"adversarial","name":"Adversarial Testing","description":"Break it intentionally from every angle.","tests":["SQL injection","XSS payloads","path traversal","command injection","empty/null inputs","oversized payloads","unicode edge cases","concurrent requests","environment chaos","protocol chaos"]}
{"kind":"category","id":"workflow","name":"Product Owner Validation","description":"Test complete user workflows, not individual functions.","focus":["User goals","not implementation","outcomes not code","end-to-end flows","actual usage","documentation examples","user stories","acceptance criteria"]}
{"kind":"check","id":"exit_codes","category":"cli","severity":"major","text":"Verify exit codes match conventions (0=success, non-zero=error).","test":"echo $? after every command","expect":"success = 0, errors = 1-255","failure":"Exit code mismatched expectation"}
{"kind":"check","id":"error_messages","category":"cli","severity":"major","text":"Error messages must be actionable, specific, and helpful.","test":"Trigger errors, read messages","expect":["context about what failed","suggestion for fix","clear not cryptic"],"failure":["cryptic error","no context","unhelpful message"]}
{"kind":"check","id":"help_completeness","category":"cli","severity":"minor","text":"Help text must document all commands and working examples.","test":"grep --help, check README","expect":["all subcommands listed","usage examples work","no TODO/placeholder text"],"failure":["missing commands","examples don't work","TODO in output"]}
{"kind":"check","id":"panic_detection","category":"all","severity":"critical","text":"ZERO tolerance for panics/todo/unimplemented in user-facing code.","test":"grep output for panic/todo/unimplemented","bans":["panic!","todo!","unimplemented!","unwrap failed"],"failure":["Panic in output","todo! in user code","unimplemented! in production"]}
{"kind":"check","id":"secret_leak","category":"all","severity":"critical","text":"Secrets must never appear in output.","test":"grep -iE 'password|token|key|secret|api_key'","bans":["password=","token=","secret=","api_key="],"failure":["Possible secret in output"]}
{"kind":"check","id":"status_codes","category":"api","severity":"major","text":"HTTP status codes must be correct for the operation.","test":"curl -w '%{http_code}'","expect":["200/201 for success","400 for bad input","401/403 for auth","404 for not found","500 only for unexpected errors"],"failure":["500 for expected errors","404 returns 500","wrong status code"]}
{"kind":"check","id":"response_schema","category":"api","severity":"major","text":"Response body must match documented schema.","test":"jq validation against schema","expect":"valid JSON/expected fields","failure":["schema mismatch","missing fields","wrong types"]}
{"kind":"check","id":"auth_required","category":"api","severity":"critical","text":"Protected endpoints must reject unauthenticated requests.","test":"curl without auth header","expect":"401/403 response","failure":["Returns 200 without auth","Returns 500 instead of 401"]}
{"kind":"check","id":"rate_limiting","category":"api","severity":"major","text":"Public endpoints must enforce rate limits.","test":"send 20+ rapid requests","expect":"429 Too Many Requests","failure":["No rate limiting","Accepts unlimited requests"]}
{"kind":"check","id":"sql_injection","category":"adversarial","severity":"critical","text":"Input must sanitize SQL injection attempts.","test":"Send \"'; DROP TABLE users; --\"","expect":"Rejected/escaped safely","failure":["SQL error","Database affected","Injection succeeded"]}
{"kind":"check","id":"xss_payload","category":"adversarial","severity":"critical","text":"Input must sanitize XSS attempts.","test":"Send \"<script>alert('xss')</script>\"","expect":"Escaped/rejected safely","failure":["Script executes","Payload reflected unchanged"]}
{"kind":"check","id":"path_traversal","category":"adversarial","severity":"critical","text":"File paths must sanitize traversal attempts.","test":"Send \"../../../../etc/passwd\"","expect":"Rejected/sandboxed","failure":["File access outside allowed","Traversal succeeded"]}
{"kind":"check","id":"user_workflow_complete","category":"workflow","severity":"critical","text":"Complete user workflow must work end-to-end.","test":"Follow documentation examples step-by-step","expect":"User achieves stated goal","failure":["Workflow breaks","Missing steps","Unclear next action"]}
{"kind":"pattern","id":"smoke_test","description":"Quick validation that basic functionality works.","steps":["1. Run basic command","2. Check exit code 0","3. Verify output non-empty","4. Check for errors"],"expect":["All pass in < 30 seconds","No crashes","Basic ops work"]}
{"kind":"pattern","id":"integration_test","description":"Test component interactions end-to-end.","steps":["1. Start services","2. Run realistic workflow","3. Verify state changes","4. Check side effects","5. Validate output"],"expect":["Complete flow works","State correct","Side effects expected"]}
{"kind":"pattern","id":"regression_test","description":"Verify previously working behavior still works.","steps":["1. Run prior test suite","2. Check for failures","3. Identify changed behavior","4. File regressions"],"expect":["All old tests pass","No behavior changes"]}
{"kind":"pattern","id":"chaos_file_operations","description":"Test file operations under adversarial conditions.","test_cases":["normal file","already exists","permission denied","disk full","invalid path","special chars in name","unicode filename","concurrent writes"],"expect":["Graceful errors","No data loss","Clear messages"]}
{"kind":"severity","id":"critical","criteria":"Crash, data loss, security issue, broken workflow","action":"Use canonical disposition blocker until repaired, then fixed_with_evidence after rerun evidence. Blocker prevents merge/sign-off.","examples":["panic in user code","SQL injection","secret leaked","workflow broken"]}
{"kind":"severity","id":"major","criteria":"Poor UX, missing validation, confusing errors, incomplete feature","action":"Use canonical disposition blocker until repaired, then fixed_with_evidence after rerun evidence. Blocker prevents merge/sign-off.","examples":["unclear error message","missing input validation","500 instead of 404","auth not enforced"]}
{"kind":"severity","id":"minor","criteria":"Suboptimal output, missing docs, performance issues","action":"Use canonical disposition fixed_with_evidence, owner_approved_debt, owner_approved_no_action, or blocker. Blocker prevents sign-off.","examples":["trailing whitespace","TODO in help","slow response","missing example"]}
{"kind":"severity","id":"observation","criteria":"Style, nice-to-have improvements","action":"Use canonical disposition owner_approved_no_action, owner_approved_debt, fixed_with_evidence, or blocker. Never ignore.","examples":["inconsistent formatting","could be clearer","optimization opportunity"]}
{"kind":"auto_fix","id":"missing_executable","pattern":"Permission denied","fix":"chmod +x file","detection":"grep -qi 'permission denied'","verification":"Re-run command, check exit 0"}
{"kind":"auto_fix","id":"trailing_whitespace","pattern":"Trailing whitespace in output","fix":"sed -i 's/[[:space:]]*$//' file","detection":"grep ' '","verification":"Re-run, check no trailing spaces"}
{"kind":"auto_fix","id":"missing_newline","pattern":"No newline at EOF","fix":"echo >> file","detection":"tail -c1 file | wc -l == 0","verification":"File ends with newline"}
{"kind":"auto_fix","id":"missing_shebang","pattern":"script fails with 'exec format error'","fix":"Add #!/usr/bin/env bash to top","detection":"error: exec format","verification":"Script executes"}
{"kind":"anti_pattern","id":"looks_good","problem":"No execution performed","correct":"Actually run it","rationale":"'Looks good' is visual inspection, not testing"}
{"kind":"anti_pattern","id":"should_pass","problem":"Hallucinated results","correct":"Run tests, report actual results","rationale":"'Should pass' means you didn't run it"}
{"kind":"anti_pattern","id":"happy_path_only","problem":"Incomplete coverage","correct":"Test error cases, edge cases, failures","rationale":"Happy path working doesn't mean quality"}
{"kind":"anti_pattern","id":"code_review","problem":"Static analysis only","correct":"Dynamic execution + inspection","rationale":"Reading code != testing behavior"}
{"kind":"gate","id":"all_tests_executed","text":"Every test must be actually executed. No skipped tests.","check":"Count tests, verify all ran","failure":"Skipped tests found"}
{"kind":"gate","id":"every_failure_has_evidence","text":"Every test failure must have command, output, exit code.","check":"Review failures, verify evidence present","failure":"Failure without evidence"}
{"kind":"gate","id":"all_findings_addressed","text":"Every finding at every severity must use canonical disposition fixed_with_evidence, owner_approved_debt, owner_approved_no_action, or blocker. Blocker prevents sign-off.","check":"Review critical, major, minor, and observation findings","failure":"Any unresolved finding remains"}
{"kind":"gate","id":"workflow_completes","text":"User workflow must complete end-to-end.","check":"Run full workflow from docs","failure":"Workflow broken"}
{"kind":"gate","id":"errors_are_actionable","text":"Error messages must be actionable and clear.","check":"Trigger errors, read messages","failure":"Unclear error messages"}
{"kind":"gate","id":"no_secrets","text":"No secrets in output.","check":"grep output for secrets","failure":"Secret leaked"}
{"kind":"gate","id":"security_passed","text":"Security tests must pass.","check":"Review adversarial tests","failure":["SQL injection","XSS","path traversal","auth bypass"]}
{"kind":"command","id":"test","usage":"qa-enforcer test <target> [--adversarial] [--deep]","description":"Run full QA test suite on target","arguments":["target: file, directory, URL, or binary","--adversarial: include chaos testing","--deep: exhaustive inspection"]}
{"kind":"command","id":"cli","usage":"qa-enforcer cli <binary>","description":"Test CLI application only","arguments":["binary: path to executable"],"output":["Exit codes","Help text","Error messages","All subcommands"]}
{"kind":"command","id":"api","usage":"qa-enforcer api <base-url> [--auth <token>]","description":"Test API endpoints only","arguments":["base-url: API base URL","--auth: optional auth token"],"output":["Status codes","Response schemas","Security checks","Rate limiting"]}
{"kind":"command","id":"workflow","usage":"qa-enforcer workflow <workflow-description>","description":"Test user workflow end-to-end","arguments":["workflow-description: user story or goal"],"output":["Workflow completion","UX quality","Blockers"]}
{"kind":"command","id":"reproduce","usage":"qa-enforcer reproduce \"<command>\"","description":"Reproduce and analyze specific issue","arguments":["command: exact command to run"],"output":["Execution result","Deep analysis","Suggested fix"]}
{"kind":"command","id":"smoke","usage":"qa-enforcer smoke <target>","description":"Quick smoke test","arguments":["target: what to test"],"output":["Basic functionality","Fast pass/fail"]}
{"kind":"example","id":"cli_basic","category":"cli","description":"Basic CLI testing workflow","code":"# Test CLI binary exists\nrun_test \"Binary exists\" \"which myapp\" 0\n\n# Test help\nrun_test \"Help works\" \"myapp --help\" 0\n\n# Test invalid input\nrun_test \"Invalid command fails\" \"myapp bogus-cmd\" 1\n\n# Test missing args\nrun_test \"Missing args shows error\" \"myapp new\" 1"}
{"kind":"example","id":"api_basic","category":"api","description":"Basic API testing workflow","code":"# Test endpoint\nresponse=$(curl -s -w \"\\n%{http_code}\" \"http://localhost:8080/api/tasks\")\nstatus=$(echo \"$response\" | tail -1)\nbody=$(echo \"$response\" | head -1)\n\n# Validate status\nif [[ ! \"$status\" =~ ^[23] ]]; then\n  fail \"Unexpected status: $status\"\nfi\n\n# Validate JSON\necho \"$body\" | jq '.'"}
{"kind":"example","id":"workflow_complete","category":"workflow","description":"End-to-end workflow test","code":"# User story: Create task and approve it\n\n# Step 1: Create\noutput=$(oya new -s test-task)\ngrep -q \"Created task\" <<< \"$output\" || fail \"Creation failed\"\n\n# Step 2: Verify\nbr show test-task || fail \"Not found\"\n\n# Step 3: Approve\noya approve -s test-task || fail \"Approval failed\"\n\n# Step 4: Verify state\nbr show test-task | grep -q \"status: approved\" || fail \"Not approved\""}
{"kind":"example","id":"adversarial_sql","category":"adversarial","description":"SQL injection test","code":"# Test SQL injection\ncurl -s -X POST \"http://localhost:8080/api/tasks\" \\\n  -d \"name='; DROP TABLE tasks; --\"\n\n# Expected: Rejected safely\n# Failure: SQL error, data loss"}
{"kind":"integration","id":"red_queen","description":"QA findings feed into red-queen lineage","workflow":["qa-enforcer test --adversarial","surivors become red-queen done_when entries","coevolution prevents regressions"]}
{"kind":"integration","id":"tcr_enforcer","description":"QA validates before TCR commits","workflow":["qa-enforcer test","if pass → tcr-enforcer commit","if fail → tcr-enforcer revert"]}
{"kind":"integration","id":"zjj","description":"Isolate fixes with zjj workspaces","workflow":["zjj add qa-fix-$(date +%s)","run qa-enforcer","fix issues","zjj done"]}
```

## When to Use QA Enforcer

Invoke this skill when you need:
- **Real CLI testing**: "Test this command thoroughly" → Actually runs it
- **API contract validation**: "Validate this endpoint" → Makes real requests
- **Adversarial testing**: "Break this" → Tries every attack vector
- **Product owner review**: "Does this solve the problem?" → Validates outcomes
- **Integration testing**: "Test the full flow" → Runs end-to-end
- **Regression hunting**: "Find what broke" → Tests everything

## Core Philosophy (JSONL-Encoded)

**QA Enforcer NEVER fakes anything.**

| What QA Does | What QA NEVER Does |
|--------------|-------------------|
| Actually runs commands and captures output | Claims "tests would pass" without running |
| Parses real stdout/stderr/exit codes | Uses AI hallucination to guess results |
| Validates actual API responses | Mocks API responses without calling |
| Tests against real contracts | Assumes contracts are correct |
| Breaks things intentionally | Only checks happy path |
| Files issues with evidence | Says "might be issues" |
| Fixes what can be fixed | Just reports problems |

The JSONL block above encodes all testing rules, patterns, checks, and examples in a mechanically-parseable format. Agents can:
- Parse test rules without markdown ambiguity
- Extract examples with exact code
- Validate compliance against encoded severity levels
- Look up auto-fix patterns by detection signature
- Reference quality gates for sign-off decisions

## Non-Negotiable Rules

### 1. Execution is Mandatory
```bash
# ❌ WRONG: AI hallucination
"The command should work because the code looks correct"

# ✅ RIGHT: Actual execution
$ oya new -s test-slug
[Captured output]
Exit code: 0
Stdout: "Created task: test-slug"
Stderr: ""
```

### 2. Evidence is Required
Every finding must include:
- **Exact command** that was run
- **Actual output** (stdout/stderr)
- **Exit code** or HTTP status
- **Expected vs Actual** comparison
- **Reproduction steps**

### 3. Deep Inspection, Not Surface Level
```bash
# ❌ SURFACE: Just checks it doesn't crash
$ myapp --help
# "Works!" - NO, you only checked one thing

# ✅ DEEP: Inspects everything
$ myapp --help
→ Verify all subcommands documented
→ Check usage examples work
→ Test hidden flags
→ Validate error messages
→ Check formatting consistency
→ Verify version flag
→ Test invalid arguments
→ Check for typos in help text
```

## Quality Gates

Before signing off, ALL must pass:

- [ ] Every test was actually executed (no skipped tests)
- [ ] Every failure has evidence (command, output, exit code)
- [ ] Every finding at every severity uses canonical disposition `fixed_with_evidence`, `owner_approved_debt`, `owner_approved_no_action`, or `blocker`; `blocker` prevents sign-off
- [ ] User workflow completes end-to-end
- [ ] Error messages are actionable
- [ ] Documentation examples work
- [ ] No secrets in output
- [ ] No panics/todo/unimplemented in user-facing code
- [ ] Security tests passed (injection, xss, etc)
- [ ] Performance is acceptable

---

**Version:** 2.0.0 (JSONL-Encoded)
**Last Updated:** February 2026
**Status:** Production Ready
**Philosophy:** Execute Everything. Inspect Deeply. Fix What You Can.
