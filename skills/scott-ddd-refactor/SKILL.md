---
name: scott-ddd-refactor
description: "Strict Scott Wlaschin style refactoring for DDD and type-driven design. Makes illegal states unrepresentable."
---

```jsonl
{"kind":"meta","skill":"scott-ddd-refactor","version":"1.0.0","mode":"strict-ddd-types","format":"jsonl-progressive","token_strategy":"top_block_plus_jsonl"}
{"kind":"mission","goal":"Refactor code so domain correctness is enforced by types, not conventions."}
{"kind":"input","arguments":"$ARGUMENTS","rule":"If scope is missing, infer from current task and scan for highest-risk domain modules first."}
{"kind":"principle","id":"illegal_states_unrepresentable","text":"Primary objective: domain models must make invalid combinations impossible to construct."}
{"kind":"principle","id":"parse_dont_validate","text":"Parse and constrain raw inputs at boundaries. Core logic accepts trusted domain types only."}
{"kind":"principle","id":"types_as_spec","text":"Function signatures and domain types are the executable specification; comments are secondary."}
{"kind":"principle","id":"explicit_workflows","text":"Business workflows are explicit state transitions, never implicit flag/nullable choreography."}
{"kind":"principle","id":"functional_core_imperative_shell","text":"Core is pure and deterministic. Side effects remain at shell boundaries."}
{"kind":"rule","id":"ban_primitive_obsession_in_domain","level":"error","text":"Domain APIs cannot use primitive placeholders for domain concepts.","bans":["String/str for identifiers","bool flags for domain decisions","u64/i64 for unitful money/quantities without semantic wrappers"],"preferred":["newtypes/value objects","semantic enums","unit-safe wrappers"]}
{"kind":"rule","id":"ban_option_as_state_machine","level":"error","text":"Do not encode workflow state through optional fields.","bans":["Option fields that imply lifecycle stage","multiple nullable fields with hidden dependencies"],"preferred":["sum types (enums)","type-state transitions"]}
{"kind":"rule","id":"ban_boolean_control_flags","level":"error","text":"No boolean parameters for behavioral branching in domain functions.","preferred":["explicit mode enums"]}
{"kind":"rule","id":"boundary_parsing_required","level":"error","text":"All untrusted input (I/O, network, DB, CLI, env) must be converted into constrained domain types before core use."}
{"kind":"rule","id":"domain_error_taxonomy","level":"error","text":"Expected domain failures must use explicit error variants; no opaque strings in core."}
{"kind":"rule","id":"exhaustive_transition_handling","level":"error","text":"State transitions must be exhaustively handled and compile-checked."}
{"kind":"anti_pattern","id":"boolean_soup","problem":"Flags and nullable fields model lifecycle implicitly","fix":"Replace with explicit state enum variants carrying only valid data"}
{"kind":"anti_pattern","id":"validation_sprinkling","problem":"Repeated ad-hoc checks throughout core","fix":"Single parse boundary + trusted types downstream"}
{"kind":"anti_pattern","id":"stringly_domain","problem":"Domain concepts represented as free-form strings","fix":"Introduce semantic newtypes with smart constructors"}
{"kind":"detector","id":"primitive_signature_scan","query":"Find domain/public functions with bool/String/i64 parameters and return migration candidates"}
{"kind":"detector","id":"state_encoding_scan","query":"Find structs with multiple Option fields or status+optional-field combos"}
{"kind":"detector","id":"revalidation_scan","query":"Find repeated format/range checks across layers for same concept"}
{"kind":"workflow","id":"strict_refactor_sequence","steps":["Inventory domain concepts and lifecycle states","Introduce semantic newtypes and smart constructors","Replace implicit state fields with explicit enums/type-state","Move parsing/validation to boundaries","Refactor workflow functions into typed transitions","Replace opaque errors with domain error taxonomy","Enforce exhaustive matches and transition tests"]}
{"kind":"decision","id":"result_vs_exception","rule":"Use Result for expected domain outcomes. Use exceptions/panics only for unrecoverable or infrastructure panic classes per runtime conventions."}
{"kind":"gate","id":"type_integrity_gate","checks":["No primitive obsession in domain interfaces","No bool control flags in domain signatures","No Option-as-state encoding","All raw input parsed before core","All domain errors are enumerable"]}
{"kind":"gate","id":"workflow_integrity_gate","checks":["State machine is explicit","Transitions are one-way where required","Illegal transitions are unconstructable","Pattern matches are exhaustive"]}
{"kind":"output_contract","sections":["1) Refactor contract (target invariants + states)","2) Typed model diffs (before/after)","3) Transition map","4) Boundary parsing plan","5) Error taxonomy","6) Test plan for transitions/invariants","7) Migration notes and risks"]}
{"kind":"ref","file":"reference.md","use":"Strict doctrine and practical interpretation"}
{"kind":"ref","file":"refactoring-playbook.md","use":"Stepwise refactor method and sequencing"}
{"kind":"ref","file":"decision-matrix.md","use":"Result vs exception and modeling choices"}
{"kind":"ref","file":"checklist.md","use":"Readiness and acceptance checklist"}
{"kind":"ref","file":"examples.md","use":"Good vs bad DDD/type refactors"}
```

Additional resources for progressive disclosure:
- [reference.md](reference.md)
- [refactoring-playbook.md](refactoring-playbook.md)
- [decision-matrix.md](decision-matrix.md)
- [checklist.md](checklist.md)
- [examples.md](examples.md)
