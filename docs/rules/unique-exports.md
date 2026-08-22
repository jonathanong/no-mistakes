# `unique-exports`

Prevents ambiguous duplicate public export names.

```yaml
rules:
  - rule: unique-exports
    projects: [web]
    options:
      uniqueAcrossTypesAndValues: true
```

Counterexample: two files in the same checked scope both export `Button`.

Next.js App Router / Pages convention exports (`metadata`, `generateMetadata`,
HTTP method handlers on `route.ts`, and similar) are exempt only inside a
detected Next.js project. Remix route-module exports (`loader`, `action`,
`clientLoader`, `clientAction`, `meta`, `links`, `ErrorBoundary`) are exempt
only for files under a configured `type: remix` project's `app/routes/` (or
`app/root.*`). A `@remix-run/*` package.json dependency is not enough.

Fix: rename one export, narrow the rule scope, or use a documented suppression
for intentional public aliases.
