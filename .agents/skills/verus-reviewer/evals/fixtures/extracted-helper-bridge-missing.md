# extracted-helper-bridge-missing

Verified helper: `verified_normalize(input@)`.
Production: `normalize(input: &str)` bypasses helper for empty strings and calls unverified parser for others. No bridge evidence.
