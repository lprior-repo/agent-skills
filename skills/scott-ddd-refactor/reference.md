# Reference

```jsonl
{"kind":"meta","doc":"reference","skill":"scott-ddd-refactor","version":"1.0.0"}
{"kind":"intent","text":"Use type-driven DDD so domain correctness is structural, not procedural."}
{"kind":"default","mode":"strict-ddd-types","text":"Dogmatic by default for AI reliability."}
{"kind":"rule","id":"domain_concept_has_type","text":"Each domain concept gets a dedicated semantic type."}
{"kind":"rule","id":"state_has_sum_type","text":"Lifecycle state is represented by enums/type-state, not nullable fields."}
{"kind":"rule","id":"boundary_parsing","text":"All untrusted input is parsed into constrained domain types at boundaries."}
{"kind":"rule","id":"pure_core","text":"Core logic is pure and deterministic; side effects remain in shell adapters."}
{"kind":"rule","id":"domain_errors_enumerable","text":"Expected domain failures use explicit error variants."}
{"kind":"migration_step","order":1,"text":"Find primitive obsession hotspots in domain/public signatures."}
{"kind":"migration_step","order":2,"text":"Introduce newtypes/value objects with smart constructors."}
{"kind":"migration_step","order":3,"text":"Replace flag/Option state encoding with explicit states."}
{"kind":"migration_step","order":4,"text":"Refactor workflows into typed transitions."}
{"kind":"migration_step","order":5,"text":"Isolate I/O and persistence in shell boundaries."}
```
