# Hands-On QA Reference

## Framework-Specific Discovery

How to discover the full interface surface for common frameworks and languages.

### Rust CLI (clap, structopt)

```bash
# Discovery
<BINARY> --help
<BINARY> -h
<BINARY> help              # clap subcommand style
<BINARY> <subcommand> --help

# Source-level discovery
grep -r "arg\|Arg::\|ArgMatches\|\.arg(" src/
grep -r "Subcommand\|subcommand\|#\[command" src/
grep -r "possible_values\|value_parser" src/
```

### Go CLI (cobra, pflag, kingpin)

```bash
# Discovery
<BINARY> --help
<BINARY> help <subcommand>
<BINARY> <subcommand> --help

# Source-level discovery
grep -r "\.Flags()\|\.Args()\|cobra\.\|pflag\." .
grep -r "\.AddCommand\|&cobra.Command" .
grep -r "Args: cobra\.\|ValidArgs" .
```

### Python CLI (argparse, click, typer)

```bash
# Discovery
python <script>.py --help
python <script>.py <subcommand> --help

# Source-level discovery
grep -r "add_argument\|argparse\|@click\|@app.command" .
grep -r "Argument\|Option\|click.option\|click.argument" .
```

### Node.js CLI (commander, yargs, oclif)

```bash
# Discovery
node <script>.js --help
npx <package> --help

# Source-level discovery
grep -r "\.command(\|\.option(\|yargs\|Command" .
```

### HTTP API Discovery

```bash
# OpenAPI/Swagger spec files
cat openapi.yaml 2>/dev/null || cat openapi.json 2>/dev/null
cat swagger.yaml 2>/dev/null || cat swagger.json 2>/dev/null

# Well-known spec endpoints
curl -sS <BASE_URL>/openapi.json 2>/dev/null | jq '.paths | keys'
curl -sS <BASE_URL>/swagger.json 2>/dev/null | jq '.paths | keys'
curl -sS <BASE_URL>/docs 2>/dev/null | head -50

# Framework-specific route patterns
grep -r "#\[route\|#\[get\|#\[post\|#\[put\|#\[delete" src/     # Rust axum/actix
grep -r "@app\.\|@router\.\|APIRouter" src/                     # Python FastAPI
grep -r "r\.GET\|r\.POST\|r\.PUT\|r\.DELETE\|router\." src/     # Go
grep -r "router\.\|app\.\|\.get(\|\.post(\|\.put(" src/          # Node Express/Koa

# GraphQL
curl -sS -X POST -H "Content-Type: application/json" \
  -d '{"query":"{ __schema { types { name } } }"}' \
  <BASE_URL>/graphql | jq '.data.__schema.types[].name'

# gRPC (if grpcurl available)
grpcurl -list <BASE_URL>
grpcurl -list <BASE_URL> <package>.<service>
```

---

## Path Category Deep Dive

### 1. Happy Paths

The baseline — does the tool work at all with correct input?

**CLI:**
- Run with all required arguments and valid values
- Run each subcommand with valid arguments
- Test with default values (no optional flags)
- Test with optional flags provided
- Test --help and --version

**API:**
- GET collection endpoint — returns 200 with expected shape
- GET single resource — returns 200 with expected fields
- POST create — returns 201 with created resource
- PUT update — returns 200 with updated resource
- DELETE — returns 204 or 200
- Health/readiness endpoint returns 200

### 2. Missing Inputs

Does the tool handle absent data gracefully?

**CLI:**
- No arguments at all
- Required argument omitted
- Required argument given but empty string `''`
- Flag present but no value `--flag` (if value expected)
- Subcommand without its required arguments
- Stdin expected but none provided

**API:**
- POST with empty body `{}`
- POST with body missing required fields
- GET/POST/PUT without auth header when auth required
- Request with Content-Length: 0
- Required query parameter omitted
- Required path parameter — test with valid but missing-style patterns

### 3. Invalid Inputs

Does the tool reject bad data with clear errors?

**CLI:**
- String where number expected: `--count abc`
- Negative where positive expected: `--count -5`
- Float where int expected: `--count 3.14`
- Invalid enum/choice value: `--format xyz`
- Nonexistent file path: `--input /no/such/file`
- Directory where file expected: `--input /tmp/`
- File where directory expected: `--output /etc/passwd`

**API:**
- Malformed JSON body: `{not json`
- Wrong content type: `Content-Type: text/plain` for JSON endpoint
- Invalid ID format: `/api/users/not-a-uuid`
- Invalid HTTP method: PATCH when only PUT supported
- Invalid query parameter type: `?limit=abc`
- Invalid JSON field types: string where number expected
- Extra unknown fields (should they be ignored or rejected?)
- Deeply nested JSON (stack depth)
- Non-UTF8 in string fields

### 4. Boundary Cases

Does the tool handle edge values correctly?

**CLI:**
- Zero values: `--count 0`, `--offset 0`, `--rate 0`
- Maximum values: `--count 999999999`, `--size max`
- Empty string: `--name ''`
- Single character: `--name a`
- Very long string: `--name "$(python3 -c "print('x'*10000)")"`
- Minimum valid input (1 item, 1 byte, etc.)

**API:**
- `?limit=0` — should return empty or error?
- `?limit=maxint` — does it cap or crash?
- `?offset=-1` — negative offset
- Empty string fields: `{"name": ""}`
- Max length strings
- Zero values in numeric fields
- Empty arrays: `{"items": []}`
- Null values: `{"field": null}`

### 5. Error Paths

Does the tool handle failure states correctly?

**CLI:**
- Nonexistent config file: `--config /nonexistent.conf`
- Unreadable file (no permissions): create file, chmod 000, try to read
- Output to unwritable directory: `--output /proc/cannot-write`
- Resource already exists: create, then create again
- Resource does not exist: delete nonexistent item
- Network unavailable: disconnect or use invalid URL
- Port already in use: start two instances

**API:**
- GET nonexistent resource: `/api/items/99999` → expect 404
- POST duplicate: create resource, create same again → expect 409
- DELETE nonexistent: expect 404 or 204
- PUT to nonexistent: expect 404 or create?
- Unauthorized: valid user, insufficient permissions → 403
- Expired auth token → 401
- Invalid auth token → 401
- Rate limit exceeded → 429 (if rate limiting exists)

### 6. Edge Cases

Does the tool handle unusual but valid situations?

**CLI:**
- Shell special characters in args: `'a;b'`, `'a$(whoami)'`, `` '`id`' ``
- Unicode: `'日本語'`, `'🎉🚀'`, `'café'`
- Null bytes in args: `--name $'hello\x00world'`
- Very long arguments (ARG_MAX limit)
- Stdin pipe: `echo data | <BINARY> process`
- Redirect from file: `<BINARY> process < input.txt`
- Signal handling: `kill -INT <pid>` during operation
- Concurrent invocations
- Running from different working directory

**API:**
- Unicode in payloads
- Concurrent identical requests (race conditions)
- Very large request body (many MB)
- Request with extra whitespace in JSON
- Request with BOM (byte order mark)
- SQL in query parameters: `?id=1 OR 1=1`
- XSS in payload: `{"name": "<script>alert(1)</script>"}`
- Path traversal: `../../../etc/passwd`
- CRLF injection in headers
- Very long URL (path + query)
- HEAD request (should it work like GET without body?)
- OPTIONS request (CORS headers present?)

---

## Test Matrix Template

Use this template to plan tests. Replace placeholders with discovered values.

```markdown
## Interface Surface Discovered

### CLI Flags
| Flag | Type | Required | Default | Values |
|------|------|----------|---------|--------|
| --<flag1> | string | yes | - | any |
| --<flag2> | int | no | 10 | 1-100 |
| --<flag3> | enum | no | auto | auto\|manual\|skip |

### Subcommands
| Subcommand | Required Args | Optional Args | Description |
|------------|---------------|---------------|-------------|
| <sub1> | --input | --output, --format | ... |
| <sub2> | --name | --force | ... |

### API Endpoints
| Method | Path | Auth | Body Schema | Description |
|--------|------|------|-------------|-------------|
| GET | <endpoint> | yes | - | List items |
| POST | <endpoint> | yes | {name, value} | Create item |
| GET | <endpoint>/:id | yes | - | Get item |
| PUT | <endpoint>/:id | yes | {name?, value?} | Update item |
| DELETE | <endpoint>/:id | yes | - | Delete item |

## Test Matrix

| ID | Category | Command/Request | Expected | Actual | Status |
|----|----------|----------------|----------|--------|--------|
| H1 | Happy | ... | ... | | |
| H2 | Happy | ... | ... | | |
| M1 | Missing | ... | ... | | |
| I1 | Invalid | ... | ... | | |
| B1 | Boundary | ... | ... | | |
| E1 | Error | ... | ... | | |
| X1 | Edge | ... | ... | | |
```

---

## Exit Code Reference

Standard exit codes to check against:

| Code | Meaning | Common In |
|------|---------|-----------|
| 0 | Success | All tools |
| 1 | General error | Most tools |
| 2 | Usage/argument error | clap, argparse |
| 126 | Command not executable | bash |
| 127 | Command not found | bash |
| 130 | SIGINT (Ctrl+C) | bash |
| 137 | SIGKILL | bash |

For API calls via curl: exit code 0 means curl succeeded (not necessarily HTTP 200). Always check HTTP status separately.

---

## HTTP Status Code Reference

| Code | Meaning | Valid When |
|------|---------|------------|
| 200 | OK | Successful GET, PUT |
| 201 | Created | Successful POST |
| 204 | No Content | Successful DELETE |
| 301/302 | Redirect | Moved resource |
| 400 | Bad Request | Invalid/malformed input |
| 401 | Unauthorized | Missing or invalid auth |
| 403 | Forbidden | Valid auth, insufficient permissions |
| 404 | Not Found | Nonexistent resource |
| 405 | Method Not Allowed | Wrong HTTP method |
| 409 | Conflict | Duplicate creation |
| 415 | Unsupported Media Type | Wrong content type |
| 422 | Unprocessable Entity | Valid JSON, invalid data |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server bug (should NEVER happen on valid input) |
| 502/503 | Gateway/Service Unavailable | Server overloaded or down |

A 500 on any valid input is always a CRITICAL finding.

---

## Adapting to Target Type

### Compiled Binary
1. Build first if needed (discovered from project files)
2. Run with full path to the binary
3. Watch for crash output: `thread 'main' panicked`, `SIGSEGV`, `Aborted`
4. Check exit codes match expectations

### Script (Python, Bash, Node)
1. Check script is executable
2. Test with explicit interpreter if needed
3. Watch for unhandled exceptions and stack traces
4. Check error messages go to stderr, not stdout

### Docker Container
1. `docker run --rm <image> --help` for discovery
2. Test with various env vars: `docker run --rm -e VAR=val <image>`
3. Test with volume mounts: `docker run --rm -v /tmp:/data <image>`
4. Check `docker logs <container>` after failed invocations

### API Server
1. Start the server if not running (discover how from project)
2. Use curl for all API calls
3. Stop the server when done if you started it

### Interactive/TUI Application
1. Test non-interactive flags (--help, --version, --config)
2. For interactive parts, note TUI testing is out of scope
3. Test any stdin/stdout pipe modes
4. Test with `yes` or piped input for prompts
