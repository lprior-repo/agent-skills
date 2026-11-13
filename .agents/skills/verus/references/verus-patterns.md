# Verus Patterns

## Core Mental Model

Verus has three modes:

| Mode | Purpose | Can call | Common output | Review hazard |
|---|---|---|---|---|
| `spec` | Mathematical definitions and predicates | `spec` | `spec fn`, `open spec fn`, `closed spec fn`, `recommends` | Treating recommendations as hard preconditions; exposing internals with `open` |
| `proof` | Ghost proof steps | `spec`, `proof` | `proof fn`, lemmas, invariants, `assert(goal) by { lemma_call(); }` | Using assumptions or broad context to force success |
| `exec` | Compiled Rust code | `spec` in contracts, `proof` in proof blocks, `exec` | Rust function with `requires`/`ensures` | Claiming runtime behavior is proved without verifying the body |

Pick mode first. Most Verus failures come from mode confusion, missing local facts, wrong integer layer, or SMT context that is too broad or too weak.

## Contract Patterns

- Use `requires` and `ensures` on `exec fn` and `proof fn`.
- Use `recommends` on `spec fn`; do not add `requires` or `ensures` to a `spec fn`.
- Write mathematical postconditions over `int`/`nat`.
- Keep executable locals in normal Rust types and prove bounds/conversions explicitly.
- Do not leak ghost-only values into executable expressions.
- Do not change contracts to make proof easier unless the contract change is explicitly in scope.
- For public specs, prefer `closed spec fn` plus lemma interface when callers should not depend on the body.

## Proof Idioms

| Idiom | Use when | Failure if missing |
|---|---|---|
| `spec fn` plus exec `ensures` | Imperative code implements math spec | Postcondition cannot connect code to math |
| `closed spec fn` plus `proof fn lemma_*` | Library abstraction should hide body | Clients cannot use hidden definition facts |
| `assert(goal) by { lemma_call(); }` | One assertion needs local facts | Ambient proof context gets huge or fact is unavailable |
| Loop invariant restating preconditions | Loop body needs outer facts | Body/postcondition cannot prove facts after loop isolation |
| Trigger on program term like `#[trigger] s[i]` | Quantified fact must instantiate at use site | Quantifier never fires or proof times out |
| `assert forall|x| ... by { ... }` | Proving a quantified postcondition needs a lemma per bound value | Bound variable is not in scope |
| `choose|x| ...` | Using an existential witness | Witness variable is unavailable |
| `reveal` / `reveal_with_fuel` | Opaque/recursive spec must unfold locally | Solver cannot see definition or over-unfolds globally |
| `by(compute)` / `by(compute_only)` | Concrete deterministic recursive computation | Proof depends on unstable SMT simplification |
| `by(nonlinear_arith)` | Local nonlinear arithmetic goal | Broad nonlinear context gets slow/flaky |
| `by(integer_ring)` | Ring-style equality/modulo fact over `int` | Unsupported inequality/division proof attempted |
| `by(bit_vector)` | Bitwise/bounded-machine arithmetic | Default integer solver cannot prove bit facts |
| `=~=` / `=~~=` | Extensional equality for collections/functions | Elementwise equality exists but collection equality fails |
| `use_type_invariant(&x)` | Need encapsulated invariant facts | Well-formedness facts are absent |
| `proof { ... }` inside exec | Exec needs proof facts for a call/postcondition | Proof-only calls leak into exec mode |

## State-Machine And Tracked Patterns

- `tracked` values represent linear ghost resources. They cannot be duplicated like ordinary ghost values.
- `state_machine!` and `tokenized_state_machine!` generate proof token types and transition/exchange functions; review generated-token use against the implementation event it represents.
- `opens_invariants`, invariants, atomic ghost wrappers, and storage protocols must preserve ownership transfer facts at the actual mutation/call site.
- For concurrency/unsafe-adjacent proof, require a bridge from abstract transition to production synchronization or memory operation.

## Anti-Patterns

- Adding `assume`, `admit`, or axiom to make a proof pass without explicit trusted-boundary approval.
- Marking production core logic `#[verifier::external_body]` or `#[verifier::external]` and reporting it as proved.
- Putting `requires` or `ensures` on a `spec fn`.
- Strengthening preconditions, weakening postconditions, or removing invariants to make a proof pass.
- Using broad proof blocks when one `assert(goal) by { lemma_call(); }` is enough.
- Triggering on arithmetic guards or artificial helper predicates instead of use-site program terms.
- Making a `closed spec fn` open instead of adding a lemma.
- Adding global fuel, module-level rlimit, or nonlinear arithmetic for one local assertion.
- Omitting loop facts because they were in the function precondition.
- Treating `cargo verus focus`, `verus --version`, or `verifies with assumptions` as fully proved evidence.
- Using `cfg(verus_only)` to change executable behavior rather than guarding ghost imports/config attributes.
- Verifying a copied model/harness and claiming the production function is proved.

## Repair Order

1. Trust marker: remove or ledger `assume`/`admit`/external/axiom before trusting any proof result.
2. Code parity: verify the artifact constrains the real production symbol.
3. Mode error: fix `spec`/`proof`/`exec`/`tracked` boundary before changing logic.
4. Contract error: fix `requires`, `ensures`, `recommends`, or integer layer.
5. Loop error: strengthen invariant and decreases relation.
6. Quantifier error: inspect trigger and use-site term; use quantifier profiling if performance is suspect.
7. Opaque/recursive error: reveal locally or use computation proof.
8. Arithmetic/bit error: localize nonlinear, integer-ring, or bit-vector escalation.
9. Collection equality error: use extensional equality.
10. Evidence error: run the exact verifier command and trust scan.
