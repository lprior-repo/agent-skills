# Verus Patterns

## Core Mental Model

Verus has three modes:

| Mode | Purpose | Can call | Common output |
|---|---|---|---|
| `spec` | Mathematical definitions and predicates | `spec` | `spec fn`, `open spec fn`, `closed spec fn`, `recommends` |
| `proof` | Ghost proof steps | `spec`, `proof` | `proof fn`, lemmas, invariants, `assert(goal) by { lemma_call(); }` |
| `exec` | Compiled Rust code | `spec` in contracts, `proof` in proof blocks, `exec` | Rust function with `requires`/`ensures` |

Pick mode first. Most Verus failures come from mixing modes, hiding necessary facts, or feeding the SMT solver the wrong shape.

## Contract Patterns

- Use `requires` and `ensures` on `exec fn` and `proof fn`.
- Use `recommends` on `spec fn` when a spec has domain expectations.
- Write mathematical postconditions over `int`/`nat`.
- Keep executable locals in normal Rust types and prove bounds/conversions explicitly.
- Do not leak ghost-only values into executable expressions.

## Proof Idioms

| Idiom | Use when | Failure if missing |
|---|---|---|
| `spec fn` plus exec `ensures` | Imperative code implements math spec | Postcondition cannot connect code to math |
| `closed spec fn` plus `proof fn lemma_*` | Library abstraction should hide body | Clients cannot use hidden definition facts |
| `assert(goal) by { lemma_call(); }` | One assertion needs local facts | Ambient proof context gets huge or fact is unavailable |
| Loop invariant restating preconditions | Loop body needs outer facts | Body/postcondition cannot prove facts after loop isolation |
| Trigger on program term like `#[trigger] s[i]` | Quantified fact must instantiate at use site | Quantifier never fires or proof times out |
| `reveal` / `reveal_with_fuel` | Opaque/recursive spec must unfold locally | Solver cannot see definition or over-unfolds globally |
| `by(compute_only)` | Concrete deterministic recursive computation | Proof depends on unstable SMT simplification |
| `by(nonlinear_arith)` | Local nonlinear arithmetic goal | Broad nonlinear context gets slow/flaky |
| `=~=` | Extensional equality for `Seq`, `Set`, `Map` | Elementwise equality exists but collection equality fails |
| `use_type_invariant(&x)` | Need encapsulated invariant facts | Well-formedness facts are absent |

## Anti-Patterns

- Adding `assume` to make a proof pass without explicit trusted-boundary approval.
- Marking code `#[verifier::external_body]` without owner/reason/expiry/compensating evidence.
- Putting `requires` or `ensures` on a `spec fn`.
- Using broad proof blocks when one `assert(goal) by { lemma_call(); }` is enough.
- Triggering on arithmetic guards instead of program terms.
- Making a `closed spec fn` open instead of adding a lemma.
- Adding global fuel/nonlinear arithmetic for one local assertion.
- Omitting loop facts because they were in the function precondition.
- Treating `verifies with assumptions` as fully proved.

## Repair Order

1. Mode error: fix `spec`/`proof`/`exec` boundary before changing logic.
2. Contract error: fix `requires`, `ensures`, `recommends`, or integer layer.
3. Loop error: strengthen invariant and decreases relation.
4. Quantifier error: inspect trigger and use-site term.
5. Opaque/recursive error: reveal locally or use computation proof.
6. Arithmetic error: localize nonlinear/integer-ring escalation.
7. Collection equality error: use extensional equality.
8. Trust shortcut: remove or label trusted-base expansion.
