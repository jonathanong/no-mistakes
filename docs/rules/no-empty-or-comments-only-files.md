# `no-empty-or-comments-only-files`

Bans tracked files that contain no executable or meaningful content.

```yaml
rules:
  - rule: no-empty-or-comments-only-files
    scope: repository
```

Counterexample: a placeholder file containing only `// TODO`.

Compliant example: a README with project-specific notes, or a source file with
an exported placeholder implementation that callers can import and test.

Fix: delete the file or add real implementation/docs content.

Suppression caveat: suppress only temporary placeholders with a `no-mistakes`
directive and a reason that names the follow-up owner or removal condition.
