# Evidence Standards

Accepted evidence:
- Exact command run.
- Working directory.
- Tool version when available.
- Exit status.
- Raw stdout/stderr or artifact path.
- Obligation ID mapping.
- Artifact path and relevant line range.

Rejected evidence:
- Conversational summary.
- Screenshot without reproducible command.
- "Looks good" review prose.
- Missing artifact.
- Tool unavailable but reported as pass.
- Waiver without owner, expiry, reason, and compensating evidence.

Approval requires all required obligations to be `PASS` by raw evidence or explicitly covered by a valid waiver.
