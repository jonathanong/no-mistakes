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

`type: remix` and `type: vitejs` only infer a project root from
`remix.config.*` / Vite+Remix or `vite.config.*`. They do not build Remix
loader/action/route-module graphs or a Vite SPA analyzer. Playwright coverage
remains Next.js App Router–oriented (`page.tsx` routes).

`type: cloudflare-workers` is a scoping label only: there is no wrangler, KV,
or Durable Object domain analyzer.

Prefer explicit `root`, `include`, `exclude`, `routes`, and queue settings over
repository conventions.

## Multiple `nextjs` projects

A repository can register more than one `type: nextjs` project (for example
two Next.js apps sharing a monorepo). `no-mistakes` resolves each into its own
frontend app — its own route root and selector roots — instead of picking
one. App-aware Playwright rules (for example `playwright-coverage`,
`playwright-unique-test-ids`, `playwright-prefer-test-id-locators`, and
`playwright-unique-html-ids`) then need each Playwright project bound to the
app it exercises; see
[Multiple frontend apps](tests.md#multiple-frontend-apps) for the binding
mechanisms and what happens when a binding is missing.
