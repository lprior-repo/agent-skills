# Checklist

```jsonl
{"kind":"meta","doc":"checklist","skill":"scott-ddd-refactor","version":"1.0.0"}
{"kind":"check","id":"topology","text":"Domain core is side-effect free; I/O remains in shell adapters"}
{"kind":"check","id":"types","text":"Domain interfaces expose semantic types, not primitives"}
{"kind":"check","id":"state","text":"Lifecycle modeled via enums/type-state, not flags/nullable coupling"}
{"kind":"check","id":"boundaries","text":"All untrusted input parsed at boundaries before entering core"}
{"kind":"check","id":"errors","text":"Expected failures are explicit domain error variants"}
{"kind":"check","id":"transitions","text":"Workflow steps are typed transitions between explicit states"}
{"kind":"check","id":"exhaustiveness","text":"No wildcard match arm in domain transition logic"}
{"kind":"check","id":"tests","text":"Transition and invariant tests cover allowed and forbidden paths"}
{"kind":"exit","text":"Ready only when illegal states are unconstructable by type design"}
```
