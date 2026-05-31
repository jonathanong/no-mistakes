# `banned-renamed-files`

Bans configured legacy filenames and reports the replacement name.

```yaml
rules:
  - rule: banned-renamed-files
    scope: repository
    options:
      banned:
        - from: middleware.ts
          to: proxy.ts
```

Counterexample: keeping both old and new filenames during a migration.

Fix: rename the file and update imports or framework references.
