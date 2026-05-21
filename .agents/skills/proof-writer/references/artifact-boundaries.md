# Artifact Boundaries

Allowed: proof/model/harness/spec/property/fuzz artifacts and proof reports. Forbidden: production behavior edits, test suite edits, weakening contracts, deleting assertions, hiding assumptions.

If proof requires production redesign, write a blocker for `holzman-rust` instead of editing production code.
