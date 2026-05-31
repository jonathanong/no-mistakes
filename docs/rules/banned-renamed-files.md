# `banned-renamed-files`

Bans configured legacy filenames and reports the replacement name.

```yaml
rules:
  - rule: banned-renamed-files
    scope: repository
    options:
      scope: web
      bannedBasenames:
        - name: middleware
          message: "rename middleware.{ts,mts,js} to proxy.ts"
      extensions: [".ts", ".mts", ".js"]
```

Counterexample: keeping both old and new filenames during a migration.

Fix: rename the file and update imports or framework references.
