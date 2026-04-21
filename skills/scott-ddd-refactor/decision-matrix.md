# Decision Matrix

```jsonl
{"kind":"meta","doc":"decision-matrix","skill":"scott-ddd-refactor","version":"1.0.0"}
{"kind":"decision","topic":"result_vs_exception","when":"Expected business/domain outcomes","choose":"Result<T, DomainError>","reason":"Expected alternatives belong in domain model"}
{"kind":"decision","topic":"result_vs_exception","when":"Unrecoverable panic or corrupted runtime assumptions","choose":"Exception/panic boundary handling","reason":"Not part of domain workflow"}
{"kind":"decision","topic":"option_vs_enum","when":"Represents lifecycle stage or branch semantics","choose":"Enum/type-state","reason":"Option hides state semantics"}
{"kind":"decision","topic":"option_vs_enum","when":"Truly optional independent attribute","choose":"Option<T>","reason":"No hidden state progression"}
{"kind":"decision","topic":"primitive_vs_newtype","when":"Identifier, constrained text, unitful number, money","choose":"Newtype/value object","reason":"Eliminate primitive obsession and improve signatures"}
{"kind":"decision","topic":"wildcard_match","when":"Domain state transitions","choose":"No wildcard arm","reason":"Compiler must force handling of new states"}
```
