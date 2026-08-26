# `agents-md-max-size`

Keeps `AGENTS.md`, `CLAUDE.md`, and similar agent instruction files within
configured line/character limits.

## What it catches

It reports each selected instruction file that exceeds `maxLines`, `maxChars`,
or both. With `advisoryCharsRemaining`, it also reports near-limit files without
failing the check.

```yaml
rules:
  - rule: agents-md-max-size
    scope: repository
    options:
      maxLines: 200
      maxChars: 12000
      advisoryCharsRemaining: 500
      filenames: [AGENTS.md, CLAUDE.md]
      roots: [.]
```

## Why and when

Use this rule when agent instruction files are part of the
repository contract and must fit reliably in an agent context window. It checks
only configured instruction filenames below the selected roots. `maxLines`
defaults to `200`, `maxChars` to `12000`, and `filenames` to `AGENTS.md` and
`CLAUDE.md`. `roots` defaults to the rule application roots; use it to limit
the search further. Omit `advisoryCharsRemaining` to disable advisory findings.

## Valid example

A short `AGENTS.md` that links to detailed repository docs rather
than repeating their contents remains below both configured limits.

Counterexample: adding a long local agent file that duplicates global policy.

Fix: move detailed reference material into docs and keep agent files concise.

`advisoryCharsRemaining` reports near-limit files without failing `no-mistakes
check`. Advisory output includes the current character count, byte count,
configured max, and remaining budget so pre-push hooks can surface context
before a hard limit failure.

## Suppression

If a file is intentionally exempted with a disable comment,
both blocking findings and near-limit advisories for this rule are suppressed.
Use suppression sparingly and prefer reducing document size where possible.

## Related rules

[`markdown-structure-budget`](markdown-structure-budget.md) can
keep large Markdown documents readable after detailed material moves out of an
agent instruction file.
