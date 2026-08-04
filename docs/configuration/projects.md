# Projects

Projects scope rules and framework analysis.

```yaml
projects:
  api:
    type: server
    root: backend
    routes: ["api/**/*.mts"]
  web:
    type: nextjs
    root: web
    include: ["app/**/*.tsx"]
    exclude: ["app/generated/**"]
```

Supported project types include `server`, `nextjs`, `remix`, `vitejs`,
`library`, `tests`, `rust`, and `cloudflare-workers`.

Prefer explicit `root`, `include`, `exclude`, `routes`, and queue settings over
repository conventions.

## Multiple `nextjs` projects

A repository can register more than one `type: nextjs` project (for example
two Next.js apps sharing a monorepo). `no-mistakes` resolves each into its own
frontend app — its own route root and selector roots — instead of picking
one. Playwright rules (`playwright-coverage`, `playwright-unique-test-ids`)
then need each Playwright project bound to the app it exercises; see
[Multiple frontend apps](tests.md#multiple-frontend-apps) for the binding
mechanisms and what happens when a binding is missing.
