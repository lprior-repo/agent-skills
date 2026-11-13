# Fixture: real vstd HashMap external body boundary

## Production source excerpt

Source: `/home/lewis/tmp/verus-modern-corpus/source/vstd/hash_map.rs:92-110`, cloned from `verus-lang/verus` for the local research corpus.

```rust
/// Returns the number of elements in the map.
#[verifier::external_body]
#[verifier::when_used_as_spec(spec_len)]
pub fn len(&self) -> (result: usize)
    ensures
        result == self@.len(),
{
    self.m.len()
}

/// Inserts the given key and value in the map.
#[verifier::external_body]
pub fn insert(&mut self, k: Key, v: Value)
    ensures
        final(self)@ == old(self)@.insert(k@, v),
{
    self.m.insert(k, v);
}
```

## Patch claim

A proof report imports `vstd::hash_map::HashMap`, calls `map.insert(k, v)`, and claims the insertion behavior is `PROVED` because `cargo verus verify -p app --no-cheating` exits 0.

## Evidence packet

```text
command: cargo verus verify -p app --no-cheating
exit status: 0
summary: 12 verified, 0 errors
trust scan: no hits in src/app.rs
```

No verifier-closure scan is attached. No trust-ledger row lists the vstd HashMap `external_body` methods. No proof-to-code mapping explains whether the application proof depends on `HashMap::insert` semantics or treats vstd collections as an imported TCB.

## Required reviewer behavior

Reject or block approval. The reviewer must cite the real source lines above, require full verifier-closure trust scanning, classify the vstd `external_body` methods as imported `TRUSTED` boundary evidence rather than application `PROVED` facts, and require a ledger/TCB entry or an alternative verified data structure if the insertion semantics are part of the claimed behavior.
