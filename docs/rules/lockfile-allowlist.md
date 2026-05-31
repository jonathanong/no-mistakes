# `lockfile-allowlist`

Allows only configured package-manager lock files.

```yaml
rules:
  - rule: lockfile-allowlist
    scope: repository
    options:
      allowed: [pnpm-lock.yaml]
```

Counterexample: adding `package-lock.json` to a pnpm workspace.

Fix: remove the unexpected lockfile or update the allowlist.
