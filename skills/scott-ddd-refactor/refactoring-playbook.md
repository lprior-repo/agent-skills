# Refactoring Playbook

```jsonl
{"kind":"meta","doc":"refactoring-playbook","skill":"scott-ddd-refactor","version":"1.0.0"}
{"kind":"phase","id":"domain_audit","order":1,"actions":["Enumerate domain terms and lifecycle states","Find primitive placeholders in domain signatures","Find flag/nullable state encodings"]}
{"kind":"phase","id":"introduce_semantic_types","order":2,"actions":["Create value objects/newtypes for IDs, money, units, constrained strings","Add smart constructors/parsers","Migrate public interfaces first"]}
{"kind":"phase","id":"explicit_state_modeling","order":3,"actions":["Replace status+Option combinations with enum variants","Ensure each variant carries only valid data","Delete impossible field combinations"]}
{"kind":"phase","id":"typed_transitions","order":4,"actions":["Define transition signatures StateA -> Result<StateB, DomainError>","Keep transitions single-purpose","Compose transitions into workflow"]}
{"kind":"phase","id":"boundary_isolation","order":5,"actions":["Keep DTO/transport/storage parsing in shell adapters","Convert boundary models into domain types before core","Prevent raw input leakage into core"]}
{"kind":"phase","id":"verification","order":6,"actions":["Add tests for valid transitions","Add tests for prohibited transitions","Enforce exhaustive pattern matches"]}
```
