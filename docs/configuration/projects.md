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
    rewrites:
      - source: /posts/:slug*
        destination: /content/posts/:slug*
```

Supported project types include `server`, `nextjs`, `remix`, `vitejs`,
`library`, `tests`, `rust`, and `cloudflare-workers`.

`type: remix` infers a project root from `remix.config.*` / Vite+Remix when
`root` is omitted. That root feeds a thin file-based `server routes` index
(`app/routes/users.$id.tsx` → `/users/:id`) and `unique-exports` exemptions
for Remix route-module exports (`loader`, `action`, `clientLoader`,
`clientAction`, `meta`, `links`, `ErrorBoundary`). Remix is not added to
`frontend_apps()`, so Playwright coverage and Next.js fetches stay Next.js App
Router–oriented (`page.tsx` routes). `type: vitejs` still only infers a
project root from `vite.config.*`; it does not build a Vite SPA analyzer.

`type: cloudflare-workers` is a scoping label only: there is no wrangler, KV,
or Durable Object domain analyzer.

Prefer explicit `root`, `include`, `exclude`, `routes`, queue settings, and
`trpc.routers` over repository conventions.

`rewrites` belongs to the project that owns the routes. Each rule maps a
browser-facing source path to the destination path used by the application;
the fetch and route analyzers use these declarations to connect those paths.
Keep rewrite globs and parameters static so the relationship remains
deterministic.

Empty `trpc.routers` lists disable tRPC procedure extraction. There is no
hardcoded `src/trpc`. Request `--relationship trpc` to traverse
`trpc-call` / `trpc-procedure` edges; they stay out of unfiltered
`dependencies` and `--relationship all`.

```yaml
projects:
  api:
    type: server
    root: backend
    trpc:
      routers: ["src/trpc/**/*.ts"]
```

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
