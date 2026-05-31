# `package-json-registry-only`

Requires package registry settings to match configured policy.

```yaml
rules:
  - rule: package-json-registry-only
    scope: repository
    options:
      scopes: [packages]
      lockfile: pnpm-lock.yaml
```

Counterexample: `package.json` using `file:`, `link:`, `git+https:`, or direct
tarball-style dependency specifiers.

Fix: use npm registry versions, `workspace:`, `catalog:`, or supported
`npm:` aliases; keep lockfile package entries registry-backed.
