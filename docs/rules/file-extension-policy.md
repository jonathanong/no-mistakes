# `file-extension-policy`

Enforces allowed or banned file extensions under configured scopes.

```yaml
rules:
  - rule: file-extension-policy
    scope: repository
    include: ["src/**"]
    options:
      allowed: [ts, tsx, mts]
```

Counterexample: adding `src/helper.js` where only TypeScript is allowed.

Fix: rename or move the file, or adjust the policy intentionally.
