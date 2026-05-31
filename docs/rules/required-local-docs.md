# `required-local-docs`

Requires documentation beside configured code directories.

```yaml
rules:
  - rule: required-local-docs
    scope: repository
    options:
      roots: [agents]
      requiredFile: README.md
      codeExtensions: [mts, ts]
```

Counterexample: `agents/email/index.mts` without `agents/email/README.md`.

Fix: add the local doc or adjust roots/excludes.
