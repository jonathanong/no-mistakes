# `agents-md-max-size`

Keeps `AGENTS.md`, `CLAUDE.md`, and similar agent instruction files within
configured line/character limits.

```yaml
rules:
  - rule: agents-md-max-size
    scope: repository
    options:
      maxLines: 200
      maxChars: 12000
      advisoryCharsRemaining: 500
```

Counterexample: adding a long local agent file that duplicates global policy.

Fix: move detailed reference material into docs and keep agent files concise.

`advisoryCharsRemaining` reports near-limit files without failing `no-mistakes
check`. Advisory output includes the current character count, byte count,
configured max, and remaining budget so pre-push hooks can surface context
before a hard limit failure.

Suppression caveat: if a file is intentionally exempted with a disable comment,
both blocking findings and near-limit advisories for this rule are suppressed.
Use suppression sparingly and prefer reducing document size where possible.
