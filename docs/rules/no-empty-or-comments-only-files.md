# `no-empty-or-comments-only-files`

Bans tracked files that contain no executable or meaningful content.

```yaml
rules:
  - rule: no-empty-or-comments-only-files
    scope: repository
```

Counterexample: a placeholder file containing only `// TODO`.

Fix: delete the file or add real implementation/docs content.
