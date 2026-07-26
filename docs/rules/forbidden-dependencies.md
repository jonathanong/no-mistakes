# `forbidden-dependencies`

Prevents configured source roots from importing forbidden files or modules.

```yaml
rules:
  - rule: forbidden-dependencies
    projects: [web]
    options:
      roots: ["web/app"]
      forbiddenModules: ["fs", "node:*"]
      relationships: [import, workspace]
```

Counterexample: client code importing a server-only package.

Compliant example: client code imports from `web/app/public-api.ts`, and that
public boundary owns any server-only implementation detail.

Fix: move the dependency behind an allowed boundary or remove the import.

`relationships` limits which graph edges participate in reachability. Use
`import` plus `workspace` for runtime import boundaries: `import` includes
static, type-only, dynamic, and `require()` imports, while `workspace` follows
imports through local workspace package entry points. Omitting `relationships`
uses every standard relationship family, including routes, tests, queues,
resources, and Playwright edges; keep that broader behavior only when the
boundary intentionally spans those domains.

Suppression caveat: suppress only with a `no-mistakes` directive and a concrete
justification, and prefer narrowing `forbiddenModules` or configured roots when
the boundary is intentionally allowed. Review suppressions during boundary
changes.
