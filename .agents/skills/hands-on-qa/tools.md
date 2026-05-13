# Testing Tool Reference

Generic recipes using tools available on this system. Replace `<PLACEHOLDER>` values with the actual target discovered during the DISCOVER phase.

## Conventions

| Placeholder | Meaning | Example |
|-------------|---------|---------|
| `<BINARY>` | Path to the CLI binary being tested | `./target/release/myapp` |
| `<BASE_URL>` | Base URL of the API being tested | `http://localhost:3000` |
| `<ENDPOINT>` | A specific API route | `/api/users` |
| `<RESOURCE>` | A specific resource instance | `/api/users/42` |
| `<SUBCOMMAND>` | A CLI subcommand | `sync`, `list`, `deploy` |

---

## curl — HTTP API Testing

**When:** Any HTTP API endpoint.

### Core Flags

| Flag | Purpose |
|------|---------|
| `-sS` | Silent + show errors |
| `-w "\nHTTP %{http_code}\n"` | Print status code after body |
| `-o /dev/null` | Discard body, just check status |
| `-D -` | Dump response headers |
| `-X METHOD` | Set HTTP method |
| `-H "Header: Value"` | Set request header |
| `-d @file` | Send file as body |
| `-d 'json'` | Send string as body |
| `--max-time N` | Timeout after N seconds |
| `-v` | Verbose (full request + response) |

### Recipes — Happy Path

```bash
# GET with status code
curl -sS -w "\nHTTP %{http_code}\n" <BASE_URL><ENDPOINT>

# POST with JSON payload
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"<field>":"<valid_value>"}' \
  <BASE_URL><ENDPOINT>

# PUT update
curl -sS -w "\nHTTP %{http_code}\n" \
  -X PUT \
  -H "Content-Type: application/json" \
  -d '{"<field>":"<updated_value>"}' \
  <BASE_URL><RESOURCE>

# DELETE
curl -sS -w "\nHTTP %{http_code}\n" \
  -X DELETE \
  <BASE_URL><RESOURCE>

# With authentication
curl -sS -w "\nHTTP %{http_code}\n" \
  -H "Authorization: Bearer <TOKEN>" \
  <BASE_URL><ENDPOINT>
```

### Recipes — Failure Paths

```bash
# Missing required field
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"<wrong_field>":"value"}' \
  <BASE_URL><ENDPOINT>

# Empty body
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST \
  -H "Content-Type: application/json" \
  <BASE_URL><ENDPOINT>

# Malformed JSON
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{not valid json' \
  <BASE_URL><ENDPOINT>

# Wrong content type
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST \
  -H "Content-Type: text/plain" \
  -d '{"<field>":"value"}' \
  <BASE_URL><ENDPOINT>

# Missing auth
curl -sS -w "\nHTTP %{http_code}\n" \
  <BASE_URL><ENDPOINT>   # omit auth header entirely

# Nonexistent resource
curl -sS -w "\nHTTP %{http_code}\n" \
  <BASE_URL><ENDPOINT>/99999

# Wrong HTTP method (e.g. PATCH when only PUT is supported)
curl -sS -w "\nHTTP %{http_code}\n" \
  -X PATCH \
  <BASE_URL><RESOURCE>

# Invalid query parameter type
curl -sS -w "\nHTTP %{http_code}\n" \
  "<BASE_URL><ENDPOINT>?<param>=notanumber"

# Boundary: zero limit
curl -sS -w "\nHTTP %{http_code}\n" \
  "<BASE_URL><ENDPOINT>?limit=0"

# Boundary: negative offset
curl -sS -w "\nHTTP %{http_code}\n" \
  "<BASE_URL><ENDPOINT>?offset=-1"

# Very long string payload
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST \
  -H "Content-Type: application/json" \
  -d "{\"<field>\":\"$(python3 -c "print('A'*10000)")\"}" \
  <BASE_URL><ENDPOINT>

# Unicode in payload
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"<field>":"日本語テスト 🎉"}' \
  <BASE_URL><ENDPOINT>

# Injection attempts
curl -sS -w "\nHTTP %{http_code}\n" \
  "<BASE_URL><ENDPOINT>?<id_field>=1%20OR%201=1"
curl -sS -w "\nHTTP %{http_code}\n" \
  "<BASE_URL>/../../../etc/passwd"

# Full verbose inspection
curl -v <BASE_URL><ENDPOINT> 2>&1

# Response timing
curl -sS -w "\nHTTP %{http_code} | Time: %{time_total}s\n" \
  <BASE_URL><ENDPOINT>
```

### Recipes — Sequential Workflow

```bash
# Create → Read → Update → Delete lifecycle
ID=$(curl -sS -X POST \
  -H "Content-Type: application/json" \
  -d '{"<field>":"<value>"}' \
  <BASE_URL><ENDPOINT> | jq -r '.id // .<id_field>')
echo "Created: $ID"

curl -sS -w "\nHTTP %{http_code}\n" <BASE_URL><ENDPOINT>/$ID | jq .
curl -sS -w "\nHTTP %{http_code}\n" \
  -X PUT -H "Content-Type: application/json" \
  -d '{"<field>":"updated"}' \
  <BASE_URL><ENDPOINT>/$ID
curl -sS -w "\nHTTP %{http_code}\n" -X DELETE <BASE_URL><ENDPOINT>/$ID
curl -sS -w "\nHTTP %{http_code}\n" <BASE_URL><ENDPOINT>/$ID
```

---

## jq — JSON Response Inspection

**When:** Parsing API responses, validating structure, extracting fields.

```bash
# Pretty-print response
curl -sS <BASE_URL><ENDPOINT> | jq .

# Extract specific field
curl -sS <BASE_URL><ENDPOINT> | jq '.<field>'

# Check response has expected keys
curl -sS <BASE_URL><ENDPOINT> | jq 'keys'

# Count items in array
curl -sS <BASE_URL><ENDPOINT> | jq '. | length'

# Validate response shape
curl -sS <BASE_URL><RESOURCE> | jq '{<field1>: .<field1>, <field2>: .<field2>}'

# Inspect error response
curl -sS -w "\nHTTP %{http_code}\n" \
  -X POST -d 'bad' -H "Content-Type: application/json" \
  <BASE_URL><ENDPOINT> | jq .

# Check for duplicate IDs
curl -sS <BASE_URL><ENDPOINT> | jq '[.[].<id_field>] | unique | length'

# Paginated response check
curl -sS "<BASE_URL><ENDPOINT>?limit=2&offset=0" \
  | jq '{total: .total, count: (.<items_field> | length)}'
```

---

## timeout — Prevent Hanging Commands

**When:** Any command that might hang (servers, commands waiting for stdin, network calls).

```bash
# Run with timeout
timeout 5 <BINARY> <args> 2>&1; echo "Exit code: $?"

# Timeout with SIGKILL after grace period
timeout -s KILL 10 <BINARY> <args> 2>&1

# Start server, test it, stop it
timeout 5 <BINARY> --serve &
sleep 2
curl -sS <BASE_URL>/health
kill %1 2>/dev/null

# Test command that should complete quickly
timeout 2 <BINARY> process --input testdata.txt 2>&1
```

---

## diff — Output Comparison and Regression

**When:** Comparing actual vs expected output, or checking if output changed between versions.

```bash
# Compare against expected output
<BINARY> <args> 2>&1 | diff - expected_output.txt

# Side-by-side for different flag values
diff --side-by-side <(<BINARY> --flag A 2>&1) <(<BINARY> --flag B 2>&1)

# Check API response consistency
diff <(curl -sS <BASE_URL><ENDPOINT>) <(curl -sS <BASE_URL><ENDPOINT>)

# Unified diff for report
<BINARY> <args> 2>&1 | diff -u expected.txt -
```

---

## file — Binary Type Inspection

**When:** Checking what kind of binary you're testing.

```bash
# Identify binary type
file <BINARY>

# Check static vs dynamic linking
file <BINARY> | grep -q "dynamically linked" && echo "dynamic" || echo "static"
```

---

## ldd — Dependency Check

**When:** Binary fails to start with "shared library not found".

```bash
# Check shared library dependencies
ldd <BINARY> 2>&1

# Quick check: any missing?
ldd <BINARY> 2>&1 | grep "not found"
```

---

## hexdump — Inspect Non-Text Output

**When:** Binary produces non-text output and you need to see what it actually is.

```bash
# Hex dump of binary output
<BINARY> --export 2>&1 | hexdump -C | head -20

# Quick check: is output text or binary?
<BINARY> --export 2>&1 | file -
```

---

## script — Full Session Capture

**When:** Capturing an entire testing session as evidence for the report.

```bash
# Record session to log file
script -q testing_session.log
# ... run all tests ...
exit

# Include the log as evidence
cat testing_session.log
```

---

## Shell Patterns — Test Data Generation

```bash
# Long string
python3 -c "print('A' * 10000)"

# Special characters
printf 'hello\tworld\nwith\nnewlines\x00null'

# Empty input
echo ""

# Very long argument
<BINARY> --<field> "$(python3 -c "print('x' * 100000)")"

# Pipe input
echo "test data" | <BINARY> <subcommand>

# Here-document for multi-line input
<BINARY> <subcommand> <<EOF
line1
line2
line3
EOF

# /dev/null as input (empty file)
<BINARY> <subcommand> < /dev/null

# /dev/urandom as input (binary garbage)
timeout 1 <BINARY> <subcommand> < /dev/urandom 2>&1

# Boundary numbers
<BINARY> --<count-field> $(python3 -c "print(2**31 - 1)")   # i32 max
<BINARY> --<count-field> $(python3 -c "print(2**63 - 1)")   # i64 max
<BINARY> --<count-field> $(python3 -c "print(-1)")           # negative
<BINARY> --<count-field> $(python3 -c "print(0)")            # zero

# Path with spaces
<BINARY> --<path-field> "/path/with spaces/file.txt"

# Path traversal
<BINARY> --<path-field> "../../../etc/passwd"
<BINARY> --<path-field> "../../../../../../etc/shadow"

# Unicode in string args
<BINARY> --<field> "日本語テスト"
<BINARY> --<field> "emoji 🎉🚀"
<BINARY> --<field> "null\x00byte"
```

---

## Environment Variable Testing

```bash
# Custom env vars
<ENV_VAR>=<value> <BINARY> 2>&1

# Missing required env var
env -u <REQUIRED_VAR> <BINARY> 2>&1

# Empty env var
<REQUIRED_VAR>="" <BINARY> 2>&1

# Invalid value
<PORT_VAR>=abc <BINARY> 2>&1
<PORT_VAR>=-1 <BINARY> 2>&1
<PORT_VAR>=99999 <BINARY> 2>&1

# Injection attempt
<PATH_VAR>="/etc/passwd" <BINARY> 2>&1
```

---

## Combining Tools

```bash
# Full API test: request + status + JSON parse (falls back to raw if not JSON)
curl -sS -w "\nHTTP %{http_code}\n" <BASE_URL><ENDPOINT> | jq . 2>/dev/null || cat

# Time a slow endpoint
time curl -sS -o /dev/null -w "HTTP %{http_code}" <BASE_URL><SLOW_ENDPOINT>

# Compare two versions
diff <(<BINARY>-v1 <args> 2>&1) <(<BINARY>-v2 <args> 2>&1)

# API test then check server stderr for errors
curl -sS -w "\nHTTP %{http_code}\n" -X POST -d 'bad' <BASE_URL><ENDPOINT>
# Check server logs/stderr next
```
