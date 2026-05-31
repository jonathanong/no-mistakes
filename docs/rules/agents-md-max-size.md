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
```

Counterexample: adding a long local agent file that duplicates global policy.

Fix: move detailed reference material into docs and keep agent files concise.
