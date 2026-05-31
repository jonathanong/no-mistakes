# `unique-exports`

Prevents ambiguous duplicate public export names.

```yaml
rules:
  - rule: unique-exports
    projects: [web]
    options:
      uniqueAcrossTypesAndValues: true
```

Counterexample: two files in the same checked scope both export `Button`.

Fix: rename one export, narrow the rule scope, or use a documented suppression
for intentional public aliases.
