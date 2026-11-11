# Editing the rules plugin

Be conservative with MUST NOT. Agents treat it as a defect to fix before
handoff, so an overbroad MUST NOT forces wrong changes in codebases where the
pattern is deliberate.

- Use MUST NOT only when the code is wrong in every context the rule names. If
  a reasonable codebase does it on purpose, use SHOULD NOT.
- Default new rules to SHOULD NOT. Promote one to MUST NOT only for a concrete
  failure that the rule's own condition guarantees, not one that depends on
  scale, deployment shape, or architecture.
- A rule that depends on a threshold ("populated", "contended", "production
  volume"), an architecture choice, or a team convention is SHOULD NOT, unless
  the rule states its condition and exceptions.
- When a MUST NOT has a known legitimate exception, write the exception into
  the rule or downgrade it. Do not leave it for the reader to infer.
- Downgrading from MUST NOT to SHOULD NOT needs no evidence. Upgrading needs
  the failure it prevents.
