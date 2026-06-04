---
name: dioxus
description: "Complete Dioxus 0.7 framework guide covering development, CDP debugging with dioxus-agent-rs, and E2E testing with Playwright."
---

# Skill: Dioxus

Complete Dioxus 0.7 framework expertise covering modern development, auto-debugging with `dioxus-agent-rs` over CDP, and E2E testing with Playwright.

## Core Principles

1. **Modern Reactivity**: Use use_signal for atomic values, use_store with #[derive(Store)] for collections. Never use use_state in 0.7.
2. **Props Pattern**: Use ReadSignal<T> for component props that receive reactive values. Accepts Signal, Memo, Resource, or primitives with auto-conversion.
3. **Component Pattern**: Always use #[component] macro. Components are memoized by default and only re-render when Props change via PartialEq.
4. **Tailwind Integration**: Dioxus 0.7 has automatic Tailwind integration. Use asset!("/assets/tailwind.css") via document::Stylesheet and run dx serve.
5. **Async Patterns**: Use use_resource for async state. Use use_server_future for hydration-safe data. Avoid waterfalls by starting all requests first.

## Quick CLI Cheat Sheet

```bash
# Start developing
dx serve

# Add Tailwind
# 1. Create tailwind.css with @import "tailwindcss"
# 2. Add asset!("/assets/tailwind.css") to root rsx!

# Convert HTML snippet to RSX
dx translate --raw "<div class='foo'>...</div>"
```

## Development Workflow

### Phase 1: Development
1. Identify state container (Signal vs Store)
2. Define props using ReadSignal
3. Implement UI with rsx! macro
4. Style with Tailwind class strings
5. Add async data with use_resource

### Phase 2: Debugging with `dioxus-agent-rs`
Prerequisites:
- Start the app with the correct compile-time base path. For Seshat: `SESHAT_BASE_PATH=/Seshat moon run :serve`.
- Use the full app URL including any base path, e.g. `--url http://127.0.0.1:8081/Seshat/`.
- Use the rebuilt agent at `~/src/dioxus-agent-rs/target/release/dioxus-agent-rs`.
- ChromeDriver is not required by default. The agent uses CDP unless `--engine dual` is requested.
- Auto-wait is enabled by default. It rejects the Dioxus rebuild shell and waits for `window.__seshatE2eReady`, `window.__dioxusReady`, `body[data-hydrated="true"]`, or stable app DOM. Use `--no-auto-wait` only for negative tests.

Commands using `dioxus-agent-rs`:
- `~/src/dioxus-agent-rs/target/release/dioxus-agent-rs --url http://127.0.0.1:8081/Seshat/ dom` - Returns full HTML of the hydrated app.
- `~/src/dioxus-agent-rs/target/release/dioxus-agent-rs --url http://127.0.0.1:8081/Seshat/ --json title` - Verifies the browser sees the app title as JSON.
- `~/src/dioxus-agent-rs/target/release/dioxus-agent-rs --url http://127.0.0.1:8081/Seshat/ eval "window.__seshatE2eReady"` - Checks app-specific readiness.
- `~/src/dioxus-agent-rs/target/release/dioxus-agent-rs --url http://127.0.0.1:8081/Seshat/ click "<css-selector>"` - Simulates a click.
- `~/src/dioxus-agent-rs/target/release/dioxus-agent-rs --url http://127.0.0.1:8081/Seshat/ screenshot "<path>"` - Captures a screenshot.

### Phase 3: E2E Testing with Playwright

#### Key Principles

**1. Always Wait for Rebuild Overlay**
Before any interaction, ensure rebuild overlay is NOT present:
```typescript
await waitForNoRebuildOverlay(page);
await page.click('[data-testid="tool-text"]');
```

**2. Use Auto-Retrying Assertions**
```typescript
// GOOD: Auto-retrying assertion
await expect.poll(() => nodeCount(page), { timeout: 5000 }).toBe(1);
```

**3. Implement Robust Helper Functions**
Every test suite should include:
- `waitForNoRebuildOverlay(page)` - Waits for Dioxus rebuild to complete
- `waitForUiReady(page)` - Waits for UI to be fully hydrated
- `waitForE2eReady(page)` - Waits for app-specific E2E hooks
- `resetDocument(page)` - Resets app state without page reload
- `waitForCleanState(page)` - Verifies clean initial state
- `freshStart(page)` - Full fresh-start sequence

**4. Handle WebServer Rebuilds**
```typescript
webServer: {
  command: "dx serve --platform web --port 8082 --watch false --hot-reload false --interactive false",
  url: "http://127.0.0.1:8082",
  reuseExistingServer: true,
  timeout: 300_000,
}
```

## Progressive Disclosure

For deep-dives into specific areas:
- **reference.md**: API signatures, type system, and core mechanics
- **fullstack.md**: Server-side rendering, streaming, and real-time
- **templates.md**: Copy-paste code for lists, auth, websockets
- **examples.md**: Learn by seeing Good vs. Bad patterns
- **migration.md**: Upgrading from 0.5/0.6 to 0.7

## Trigger Language

Activate when:
- User mentions Dioxus, dioxus, signals, stores, rsx, tailwind, fullstack, router
- E2E testing for Dioxus apps
- WebDriver debugging for Rust web apps
- Playwright configuration for Dioxus

```jsonl
{"kind":"meta","skill":"dioxus","version":"1.0.0","format":"markdown-with-embedded-jsonl","mode":"contract-first"}
{"kind":"input","arguments":"$ARGUMENTS","rule":"Infer target from current request context. Trigger on dioxus, signals, stores, rsx, tailwind, fullstack, router, playwright, webdriver."}
{"kind":"mission","goal":"Guide developers in building, debugging, and testing modern Dioxus 0.7 applications."}
{"kind":"rule","id":"state_management","text":"Use use_signal for atomic values, use_store with #[derive(Store)] for collections/nested state. Never use use_state in 0.7."}
{"kind":"rule","id":"props_pattern","text":"Use ReadSignal<T> for component props that receive reactive values."}
{"kind":"rule","id":"component_pattern","text":"Always use #[component] macro. Components are memoized by default."}
{"kind":"rule","id":"tailwind_integration","text":"Dioxus 0.7 has automatic Tailwind integration. Use asset!(\"/assets/tailwind.css\") via document::Stylesheet and run dx serve."}
{"kind":"rule","id":"async_patterns","text":"Use use_resource for async state. Use use_server_future for hydration-safe data. Avoid waterfalls."}
{"kind":"rule","id":"e2e_wait_for_rebuild","text":"Always waitForNoRebuildOverlay before any Playwright interaction or assertion."}
{"kind":"tool","name":"dioxus-agent-rs","path":"~/src/dioxus-agent-rs/target/release/dioxus-agent-rs","desc":"Pure-Rust CDP-first CLI for Dioxus automation. Auto-waits for hydrated app DOM by default and rejects rebuild shell pages."}
{"kind":"workflow","id":"dioxus_dev","steps":["Identify state container (Signal vs Store)","Define props using ReadSignal","Implement UI with rsx! macro","Style with Tailwind class strings","Add async data with use_resource"]}
{"kind":"cmd","group":"cli","commands":{"dx new":"Create new project","dx serve":"Start dev server with hot-reload","dx bundle":"Build for production","dx translate":"Convert HTML to RSX"}}
{"kind":"cmd","group":"debugging","commands":{"dioxus-agent-rs --url URL dom":"Read hydrated UI state","dioxus-agent-rs --url URL screenshot PATH":"Take screenshot","dioxus-agent-rs --url URL click SELECTOR":"Click element","dioxus-agent-rs --url URL --no-auto-wait dom":"Bypass readiness wait for negative tests"}}
{"kind":"ref","file":"reference.md","use":"API signatures and core mechanics"}
{"kind":"ref","file":"fullstack.md","use":"Server functions and streaming"}
{"kind":"ref","file":"templates.md","use":"Ready-to-use boilerplate"}
{"kind":"ref","file":"examples.md","use":"Good vs Bad comparisons"}
{"kind":"ref","file":"migration.md","use":"Upgrading from older versions"}
```

## Common Issues and Solutions

### Issue: Node creation failing intermittently
**Cause**: Test is not waiting for rebuild after node creation
**Solution**: Use auto-retrying assertions with waitForNoRebuildOverlay

### Issue: Test times out waiting for element
**Cause**: Rebuild overlay is blocking interaction
**Solution**: Always call waitForNoRebuildOverlay before interaction

### Issue: Agent sees Dioxus rebuild shell
**Cause**: The app URL is wrong, the app is still rebuilding, or the compile-time base path is missing
**Solution**: Start with the correct base path, pass the full base-path URL to `--url`, and keep default auto-wait enabled

### Issue: Scene loading fails
**Cause**: Not waiting for rebuild after scene import
**Solution**: Add waitForNoRebuildOverlay after import operations
