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

Compliant example: client code imports from `web/app/public-api.ts`, and that
public boundary owns any server-only implementation detail.

Fix: move the dependency behind an allowed boundary or remove the import.

Suppression caveat: suppress only with a `no-mistakes` directive and a concrete
justification, and prefer narrowing `forbiddenModules` or configured roots when
the boundary is intentionally allowed. Review suppressions during boundary
changes.
