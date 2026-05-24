# agent-skills

Portable AI-agent skills for OpenCode, Claude Code, and compatible agent runtimes.

This repository is a checked-in snapshot of local `.agents` and OpenCode skill/agent runtime files. Within this repository, `.agents/skills/` is authoritative over the legacy top-level `skills/` mirror.

## What Is In This Repo

```text
agent-skills/
├── .agents/
│   ├── skill-improvement-plan.md
│   ├── skills/
│   │   ├── go-skill/
│   │   ├── proof-planner/
│   │   ├── proof-writer/
│   │   ├── proof-reviewer/
│   │   ├── evidence-packaging/
│   │   └── ...
│   └── tmp/
├── .opencode/
│   ├── agent/
│   └── skill/
├── skills/
├── LICENSE
└── README.md
```

The top-level `skills/` directory is preserved for compatibility with older layouts. New consumers should prefer `.agents/skills/`.

Local-only search indexes, session telemetry, and other machine-private runtime state are intentionally not included.

## Skill Format

Each skill is a directory containing a `SKILL.md` file and, when needed, supporting reference files.

Most `SKILL.md` files use:

- YAML frontmatter for runtime metadata like `name`, `description`, `allowed-tools`, and invocation controls.
- JSONL-style operational rules for compact, deterministic agent behavior.
- A `Mandatory Verification Gate` section for commands or checks the skill must prove before claiming success.
- An `Anti-Hallucination Shield` section that forbids fake outputs, invented files, and summary-only evidence.

## Install

Install into a global `.agents` runtime:

```bash
mkdir -p ~/.agents
cp -R .agents/* ~/.agents/
```

Install only the skills into a global `.agents` runtime:

```bash
mkdir -p ~/.agents/skills
cp -R .agents/skills/* ~/.agents/skills/
```

Install into Claude Code:

```bash
mkdir -p ~/.claude/skills
cp -R .agents/skills/* ~/.claude/skills/
```

Install into OpenCode:

```bash
mkdir -p ~/.opencode/skill
cp -R .agents/skills/* ~/.opencode/skill/
```

Install the OpenCode routing snapshot when using Go-skill/femdation subagents:

```bash
mkdir -p ~/.opencode/agent ~/.opencode/skill
cp -R .opencode/agent/* ~/.opencode/agent/
cp -R .opencode/skill/* ~/.opencode/skill/
```

## How To Choose A Skill

Use the skill whose boundary owns the decision.

| Need | Start with |
| --- | --- |
| Deliver a bead end to end | `go-skill` |
| Deliver many beads concurrently | `femdation` |
| Map a codebase before proof or implementation | `explore` |
| Turn a document or architecture spec into beads | `doc-to-beads` or `arch-spec-to-beads` |
| Write requirements, contracts, and proof obligations | `rust-contract` |
| Plan, write, or review formal proof artifacts | `proof-planner`, `proof-writer`, `proof-reviewer` |
| Execute proof obligations and verifier lanes | `formal-verifier` |
| Implement or repair Rust code | `holzman-rust` or `functional-rust` |
| Write or review tests | `test-planner`, `test-writer`, `test-reviewer` |
| Audit truth and evidence | `truth-serum` or `evidence-packaging` |
| Land finished work safely | `landing-skill` |
| Work on Dioxus or Makepad UI | `dioxus`, `dioxus-qa`, or the `makepad-2.0-*` skills |
| Customize Linux desktop or Omarchy config | `omarchy` |

## Primary Delivery Pipeline

The high-assurance Rust delivery flow is centered on `go-skill`.

`go-skill` does not write production code itself. It supervises specialists, verifies artifacts on disk, and blocks landing unless raw evidence supports the result.

The current proof-first lifecycle is:

```text
explore -> contract -> proof plan -> proof write -> proof review -> test plan -> test write -> test review -> implementation -> formal execution -> black-hat review -> truth-serum evidence -> landing
```

Core principle:

```text
AI agents do the work. Deterministic tools, adversarial reviewers, raw command evidence, and truth-serum decide whether the work is acceptable.
```

## Skill Catalog

### Delivery, Beads, And Orchestration

| Skill | What it does | Use it when |
| --- | --- | --- |
| `beads` | Defines the autonomous execution doctrine for the `beads` issue tracker. | You need bead lifecycle rules, issue hygiene, or bead-driven execution discipline. |
| `explore` | Scouts a codebase and writes bead-local scope artifacts without modifying production code. | You need files, APIs, crates, dependencies, risks, or existing verification artifacts mapped before planning. |
| `go-skill` | Supervises a full bead through isolation, explore, contracts, proof lifecycle, tests, implementation, verification, evidence, landing, and cleanup. | You are starting or resuming real bead delivery. |
| `femdation` | Dispatches multiple beads concurrently through the current `go-skill` lifecycle while preserving main-thread context. | You need throughput across several independent beads. |
| `evidence-packaging` | Builds the final assurance bundle and requires active-context Truth Serum before landing. | Formal execution, tests, and black-hat review are done and you need requirement-to-evidence proof. |
| `landing-skill` | Runs session completion discipline: quality gates, merge/sync/push, bead closure, orphan cleanup, and handoff. | Work is accepted and must reach main/remote without being lost locally. |

### Architecture, Planning, And Domain Modeling

| Skill | What it does | Use it when |
| --- | --- | --- |
| `arch-design-qa` | Acts as a ruthless architecture product owner using Double Diamond discovery and Munger-style mental models. | Requirements, domain model, invariants, or failure modes are still fuzzy. |
| `arch-spec-to-beads` | Takes an existing `architecture-spec.md`, runs decomposition, and persists validated beads. | A written architecture spec needs to become executable work items. |
| `architectural-drift` | Checks oversized files, cohesion drift, and DDD boundary erosion. | Code shape is decaying or refactoring needs an architectural audit. |
| `decomposer` | Shreds architecture specs into molecular tasks through the plan-shredder loop. | A large spec must become smaller sequenced work. |
| `doc-to-beads` | Reads an existing document, derives an architecture spec, and turns it into beads. | You have a design doc, brief, or proposal and want tracked work created from it. |
| `planner` | Produces deterministic atomic bead plans using the enhanced planning template. | You need clean bead decomposition with acceptance criteria and dependencies. |
| `plan-shredder` | Attacks task plans for missing constraints, hidden coupling, vague outcomes, and weak decomposition. | You need a plan stress-tested before execution. |
| `scott-ddd-refactor` | Applies Scott Wlaschin-style type-driven design so illegal states become unrepresentable. | Domain logic relies on primitives, booleans, options, or validation instead of types. |

### Contracts, Proofs, And Formal Verification

| Skill | What it does | Use it when |
| --- | --- | --- |
| `rust-contract` | Produces contracts, invariants, traceability, TLA+/Verus-first proof obligations, and BDD plans before implementation. | Behavior must be specified before code or tests are written. |
| `contract-verification-reviewer` | Independently approves or rejects contract and verification-layer artifacts before tests or implementation. | You need to know whether the contract/proof plan is strong enough to build against. |
| `proof-planner` | Chooses risk-triggered verifier lanes and writes proof obligation planning artifacts. | You need to decide whether TLA+, Verus, Kani, Flux, Loom, Miri, proptest, fuzz, or CI gates matter. |
| `proof-writer` | Writes verification artifacts only: TLA+ specs, Verus proofs, Kani harnesses, Flux refinements, Loom models, Miri checks, proptest properties, and fuzz targets. | A reviewed proof plan needs concrete proof/model/harness files. |
| `proof-reviewer` | Ruthlessly rejects weak, vacuous, unmapped, or under-executed proof artifacts. | Proof artifacts exist and need adversarial review before tests, implementation, or landing. |
| `formal-verifier` | Executes approved proof-obligation ledgers and records PASS, FAIL_LOCAL, FAIL_REGRESSION, WAIVED, or DEFERRED_GLOBAL evidence. | Proof obligations are approved and need actual verifier execution. |
| `tla-plus` | Writes, reviews, and repairs TLA+/PlusCal specs, TLC models, invariants, liveness, and counterexample evidence. | The system has temporal behavior, protocols, schedulers, queues, retries, leases, or distributed workflows. |
| `verus` | Engineers Verus specs and proofs with verifier-in-the-loop evidence and trusted-boundary hygiene. | Rust-local functions, invariants, state transitions, or proof obligations need Verus. |
| `kani` | Designs and triages Kani bounded model checking harnesses for Rust execution paths. | You need bounded proof of arithmetic, indexing, parser, codec, state-machine, panic, or unsafe precondition behavior. |
| `flux-rs` | Uses Flux refinement types for Rust invariants, signatures, predicates, and trusted boundary review. | Type-level refinements can make data constraints mechanically checked. |
| `loom` | Models Rust concurrency interleavings with Loom and related schedule exploration. | Atomics, locks, cancellation, task coordination, or concurrent state machines need deterministic schedule pressure. |
| `miri` | Runs and interprets Miri for unsafe Rust, provenance, aliasing, invalid values, leaks, and UB diagnostics. | You need UB detection evidence, especially around unsafe-adjacent or platform-sensitive Rust. |

### Rust Engineering, Performance, And Build Discipline

| Skill | What it does | Use it when |
| --- | --- | --- |
| `async-rust-reviewer` | Reviews async Rust for spawn discipline, cancellation safety, Send/Sync hygiene, stream usage, region-owned tasks, and observability. | Tokio, futures, streams, spawned tasks, or async API design may be wrong. |
| `functional-rust` | Generates or repairs Rust with functional-core discipline, zero-panic policy, and NASA/JPL reliability rules. | Rust code needs strict safety, explicit errors, and clean data-calc-action layering. |
| `holzman-rust` | Implements or optimizes Rust under Power of Ten plus performance, no-panic, bounded-resource, and evidence-first rules. | Production Rust, hot paths, CI repair, or safety-critical implementation needs a specialist. |
| `moon-v2` | Designs Moon v2 build tasks, CI caching, and Rust quality gates. | You need Moon task layout, CI/CD setup, or cache-aware verification workflows. |
| `velocity` | Pushes high-throughput delivery while keeping TDD, functional core, CI, and modern engineering discipline intact. | You need faster execution without dropping quality gates. |

### Testing, QA, And Adversarial Review

| Skill | What it does | Use it when |
| --- | --- | --- |
| `bdd-enforcer` | Ensures behavior has executable Given/When/Then scenarios and fixes missing scenario coverage. | Implemented behavior lacks end-to-end BDD proof. |
| `black-hat-reviewer` | Acts as a hardline gatekeeper for contract parity, Farley constraints, Holzman Rust, strict DDD, and simplicity. | You want ruthless review before trusting a design or implementation. |
| `hands-on-qa` | Manually invokes real CLIs, APIs, or UIs and reports only command-backed evidence. | You need to know whether a workflow actually works for a user. |
| `qa-enforcer` | Executes ruthless product QA with real commands, APIs, and deep result inspection. | You need behavior validated beyond happy-path tests. |
| `red-queen` | Evolves tests and code adversarially through deterministic state-machine pressure. | You explicitly want Digital Red Queen-style evolutionary QA. |
| `rust-fuzzer` | Designs, reviews, runs, and triages safe Rust fuzz campaigns with cargo-fuzz, AFL++, honggfuzz, LibAFL, fuzzcheck, sanitizers, and language-tooling oracles. | Rust parsers, compilers, interpreters, VMs, bytecode, JITs, or structured inputs need coverage-guided fuzzing without writing unsafe code. |
| `test-planner` | Writes exhaustive Rust test plans covering unit, BDD, proptest, mutation, and related layers. | Tests need a strategy before code is written. |
| `test-writer` | Writes exhaustive Rust tests across unit, integration, proptest, and Kani-oriented coverage. | An approved test plan needs executable tests. |
| `test-reviewer` | Reviews test plans and suites for contract parity, assertion strength, determinism, and mutation value. | You need to reject weak, tautological, or flaky tests. |
| `truth-serum` | Audits AI-generated work for hallucinations, missing evidence, deleted tests, weak verification, and runtime panic surface. | You need a zero-trust audit with active command evidence. |

### Agent Tooling, Version Control, And Operations

| Skill | What it does | Use it when |
| --- | --- | --- |
| `dolt` | Operates and diagnoses Dolt-backed bead data, remotes, and corruption scenarios. | `bd` or Dolt-backed issue state is failing. |
| `gastown` | Covers Gas Town rigs, polecats, convoys, runtimes, bead tracking, mail/nudges, formulas, and cross-rig coordination. | You are operating Gas Town or multi-agent work dispatch. |
| `jj` | Guides Jujutsu workflows, workspaces, rebasing, revsets, and GitHub/Gerrit integration. | Version control work needs `jj` rather than raw git habits. |
| `omarchy` | Handles end-user Linux desktop and window-manager customization for Hyprland, Waybar, Walker, terminals, themes, screenshots, idle, lock screen, and Omarchy commands. | You are changing desktop config, not developing Omarchy source code. |
| `opencode` | Explains OpenCode sessions, agents, providers, MCP servers, config, server mode, and GitHub integration. | You are configuring or operating OpenCode. |
| `opencode-scheduler` | Creates, inspects, updates, runs, troubleshoots, and cleans recurring OpenCode jobs. | You need scheduled agent work or scheduler diagnostics. |
| `rtk` | Installs, verifies, initializes, and diagnoses Rust Token Killer integrations across supported AI tools. | You need token-rewrite hooks or RTK wiring checked. |
| `skill-writer` | Designs, hardens, evaluates, and packages portable agent skills with contract-first behavior. | You are creating or refactoring skills. |

### Dioxus UI

| Skill | What it does | Use it when |
| --- | --- | --- |
| `dioxus` | Provides Dioxus 0.7 framework guidance for development, CDP debugging, and Playwright E2E testing. | You are building or debugging a Dioxus app. |
| `dioxus-qa` | Performs evidence-backed QA for Dioxus apps through headless Chrome and DOM validation. | Dioxus routing, UI components, or DOM updates need real browser testing. |

### Makepad 2.0 UI

| Skill | What it does | Use it when |
| --- | --- | --- |
| `makepad-2.0-animation` | Covers Makepad 2.0 animation states, timelines, transitions, hover effects, easing, and loops. | You need animation behavior or animator state fixed. |
| `makepad-2.0-app-structure` | Covers app startup, `app_main!`, event wiring, hot reload, media plugins, audio, wasm, and Cargo setup. | You are creating or restructuring a Makepad app. |
| `makepad-2.0-design-judgment` | Gives first-pass Makepad architecture and design judgment before loading narrower Makepad skills. | You need the right Makepad approach before implementation. |
| `makepad-2.0-dsl` | Explains Makepad DSL syntax, `script_mod!`, property syntax, merge operators, and widget registration. | Syntax, live design, or property definitions are wrong. |
| `makepad-2.0-events` | Covers event/action handling, callbacks, clicks, keyboard/focus events, and `MatchEvent`. | User interaction or callback behavior needs implementation or debugging. |
| `makepad-2.0-layout` | Covers layout, sizing, flow, padding, margin, alignment, scroll views, `Fill`, and `Fit`. | UI layout, spacing, alignment, or scrolling is broken. |
| `makepad-2.0-migration` | Guides migration from Makepad 1.x to 2.0 and fixes legacy API usage. | Old Makepad code needs upgrading. |
| `makepad-2.0-performance` | Covers Makepad performance, profiling, draw batching, memory, GC, and rendering speed. | UI performance or memory behavior needs evidence-backed optimization. |
| `makepad-2.0-shaders` | Covers shaders, `draw_bg`, `Sdf2d`, GPU drawing, uniforms, instances, and pixel shader behavior. | Custom drawing or shader behavior is needed. |
| `makepad-2.0-splash` | Covers Splash scripting, hot reload, `script_mod`, runtime scripting, and script integration. | Runtime scripting or hot-reload behavior is involved. |
| `makepad-2.0-theme` | Covers theming, colors, dark mode, typography, fonts, styles, and design systems. | Visual system, colors, or typography need design-system work. |
| `makepad-2.0-troubleshooting` | Covers Makepad errors, debugging, widget visibility, broken layouts, and FAQ-style diagnosis. | A Makepad app is not working and needs debugging. |
| `makepad-2.0-vector` | Covers vector graphics, SVG, gradients, paths, shapes, and tweens. | Vector drawing or animated shapes are needed. |
| `makepad-2.0-widgets` | Catalogs Makepad widgets like View, Button, Label, TextInput, CheckBox, DropDown, and Slider. | You need the right UI control or component pattern. |

## Maintenance Notes

When refreshing the snapshot from a local machine, use a non-deleting sync from your runtime skill source and keep machine-private runtime state out of the repository:

```bash
rsync -a ~/.agents/skills/ .agents/skills/
```

Recommended checks after refresh:

```bash
git status --short
git diff --stat
```

## License

MIT
