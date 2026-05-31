# `forbidden-dependencies`

Prevents configured source roots from importing forbidden files or modules.

```yaml
rules:
  - rule: forbidden-dependencies
    projects: [web]
    options:
      roots: ["web/app"]
      forbiddenModules: ["fs", "node:*"]
```

Counterexample: client code importing a server-only package.

Fix: move the dependency behind an allowed boundary or remove the import.
