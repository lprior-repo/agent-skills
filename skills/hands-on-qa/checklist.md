# Hands-On QA Checklist

## Pre-Flight Checks

Run these BEFORE starting the test plan. If any fails, report the blocker and do not proceed.

### CLI Pre-Flight

```bash
# Binary exists and is executable
ls -la <BINARY> 2>&1
file <BINARY> 2>&1

# Binary actually runs
<BINARY> --version 2>&1
<BINARY> --help 2>&1 | head -20

# Dependencies satisfied
ldd <BINARY> 2>&1 | grep "not found"
```

### API Pre-Flight

```bash
# Server is reachable
curl -sS -o /dev/null -w "HTTP %{http_code}" <BASE_URL>/ 2>&1 || echo "UNREACHABLE"

# Health endpoint (if exists)
curl -sS -w "\nHTTP %{http_code}\n" <BASE_URL>/health 2>/dev/null || echo "No /health endpoint"

# OpenAPI spec available (optional but helps discovery)
curl -sS <BASE_URL>/openapi.json 2>/dev/null | jq '.info' 2>/dev/null || echo "No OpenAPI spec"
```

---

## Coverage Verification

### CLI Coverage

Before delivering the report, verify every item:

- [ ] Every flag from `--help` has at least one happy path test
- [ ] Every flag has at least one failure path test
- [ ] Every subcommand has at least one happy path test
- [ ] Every subcommand has at least one missing-argument test
- [ ] Every required argument has a missing-input test
- [ ] Every enum/choice argument has an invalid-choice test
- [ ] At least one boundary test per numeric argument
- [ ] At least one special character test for string arguments
- [ ] `--help` outputs successfully (exit 0)
- [ ] `--version` outputs successfully (exit 0)
- [ ] Exit codes verified: success = 0, usage error ≠ 0
- [ ] Error messages appear on stderr, not stdout

### API Coverage

- [ ] Every endpoint has at least one happy path test
- [ ] Every endpoint has at least one failure path test
- [ ] Every required field has a missing-field test
- [ ] Every HTTP method for each route is tested
- [ ] Authentication paths tested (valid, missing, invalid, expired)
- [ ] Authorization paths tested (insufficient permissions)
- [ ] Error responses have correct HTTP status codes
- [ ] Error responses include actionable messages
- [ ] Response body schema validated for happy paths
- [ ] Content-Type header correct in responses
- [ ] A 500 on any valid input is flagged as CRITICAL

### Cross-Cutting Checks

- [ ] Environment variable handling (missing, empty, invalid)
- [ ] Config file handling (missing, malformed, wrong permissions)
- [ ] Signal handling (SIGINT during operation)
- [ ] Concurrent invocations (race conditions)
- [ ] Large input handling (doesn't hang or crash)
- [ ] Unicode/special character handling

---

## Report Completeness

- [ ] **Target section**: Binary/API path, version, how it was built/found
- [ ] **Interface Surface**: Every flag, subcommand, endpoint, HTTP method listed
- [ ] **Test Matrix**: An entry for every test executed with expected vs actual
- [ ] **Findings**: Each has severity, category, description, reproduction steps, evidence
- [ ] **Developer Toolbox**: Tool suggestions and commands for developer follow-up
- [ ] **Summary**: Total tests, PASS/FAIL counts, severity breakdown
- [ ] No test result based on assumption or code reading alone
- [ ] Every CRITICAL and MAJOR finding has a reproduction command the developer can re-run
- [ ] Recommended next steps are actionable (specific, not vague)

---

## Gate Checklist

Must all pass before report is delivered:

- [ ] **Surface Complete**: Every discovered interface element is tested
- [ ] **Evidence Backed**: Every PASS and FAIL has verbatim command output
- [ ] **No Assumptions**: No results from code reading alone
- [ ] **Fail Fast**: CRITICAL happy path failures reported immediately, not buried at end
