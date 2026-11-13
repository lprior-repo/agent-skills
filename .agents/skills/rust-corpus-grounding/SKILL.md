---
name: rust-corpus-grounding
description: "Ground Rust code writing/review in the 66-book corpus (rules, page-cited lookup, AST detectors)"
---

# Rust corpus grounding

Use whenever writing, reviewing, or refactoring Rust, or when you are about to
claim "the book says…". The corpus is authoritative; your recollection is not.

## Locate the corpus

Prefer a local clone; ask the user for the path if it is absent:

```
~/src/books            # or wherever lprior-repo/books is cloned
```

Key files:

| Path | Use |
|---|---|
| `BEST_PRACTICES.md` | the 40 rules, with ids and detectors — read before coding |
| `rules/rust-rules.yml` | machine form: `id, sev, statement, detect, source` |
| `dataset/corpus/<book>.md` | page-anchored text (`<!-- page: N -->`) |
| `dataset/chunks.jsonl` | retrieval chunks with page ranges |
| `code/<book>/*.rs` | extracted examples, page-anchored filenames |
| `ast-grep/rules/*.yml` | mechanical detectors |

## Procedure

1. **Read the rules.** Skim `BEST_PRACTICES.md`; keep the ids in mind
   (`ERR-1` unwrap, `UNS-1` SAFETY comment, `CON-1` ordering, `PERF-1` measure, …).
2. **Lookup when uncertain** (semantics, ordering, ownership edge cases,
   unsafe invariants, perf technique):

   ```bash
   make -C ~/src/books lookup Q="acquire release ordering"
   make -C ~/src/books lookup Q="split_at_mut" BOOK=rustonomicon
   ```

   Or without make:

   ```bash
   python3 ~/src/books/tools/lookup.py "ordering acquire" --limit 3
   ```

3. **Cite precisely.** Use the `book slug` and `pp.<a>-<b>` from the lookup
   header. Never invent page numbers or quotes. If lookup returns nothing, say
   so and fall back to the Rust Reference or the language docs.
4. **Implement** following the rules. Prefer patterns from `code/<book>/`
   examples over inventing new ones.
5. **Detect**:

   ```bash
   ~/src/books/tools/scan.sh <crate>/src
   ```

   Test/bench/fuzz trees are excluded by default; `--all` includes them.
6. **Verify**: `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`,
   tests, plus the domain gate — Miri for unsafe, loom for atomics, fuzz for
   parsers, criterion for hot paths, Kani for bounds/overflow invariants.

## Reporting

- Report detector hits as `RULE-ID file:line — statement`.
- `sev: must` blocks; `should` needs a stated reason to waive; `consider` is advisory.
- Never restate the same rule for every occurrence.

## Boundaries

- Detectors are heuristics (~10–15% false positives); verify before deleting code
  to satisfy one.
- The corpus is copyrighted. Do not paste large excerpts into public output;
  cite book + page instead. Keep repositories containing the corpus private.
