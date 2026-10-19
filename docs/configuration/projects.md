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

## PostgreSQL scope

Optional `postgres` on a project declares one PostgreSQL scope for every
enabled `postgres-*` rule that lists that project, and for
`effects.postgres.project`. The project's existing `root`, `include`, and
`exclude` already bound which files are scanned. `postgres` adds SQL locations
and executor targets. It does not infer `backend/` or scan the repository by
default.

```yaml
projects:
  db:
    root: backend
    include: [services/**, data-stores/**]
    exclude: ["**/*.test.*"]
    postgres:
      sqlInclude: [data-stores/psql/views/**/*.sql]
      schemaCatalogPath: data-stores/psql/schema-snapshot/no-mistakes-catalog.json
      unanalyzableSql: ignore
      executors:
        importSpecifier: "@data-stores/psql"
        factoryNames: [beginTransaction]
        typeNames: [TransactionQuery]
        trustedSqlTags: [{ module: "@data-stores/psql", name: sql }]
```

`sqlInclude` and `schemaCatalogPath` are project-root-relative, same as
`include`. Config load prefixes `root` when it copies them onto a rule, because
rule `options.sqlInclude`, `options.include`, and `schemaCatalogPath` are
repository-relative. `importSpecifier` and `unanalyzableSql` are not paths.
Every key is optional. An embedded-SQL rule still needs a non-empty
`executors.importSpecifier` on the project; an empty specifier does not select
every `.query` call. `factoryNames` and `typeNames` are the only executor lists
copied. Rule options still default executor names to `query`, `read`, and
`write` when `importSpecifier` is set and `executorNames` is empty. Effects do
not gain `functions` or `targets` from that default.

On config load, each enabled `postgres-*` rule whose `projects` list references
at least one project with a `postgres` block inherits that block:

- Two referenced `postgres` blocks must agree on `importSpecifier`, factory
  names, type names, trusted tags, `schemaCatalogPath`, `unanalyzableSql`, and
  `sqlInclude`. Disagreeing blocks are a config error. A rule cannot mix a `postgres` project
  with a project that has no `postgres` block.
- These rule option keys are a config error even when the value matches the
  project: `importSpecifier`, `executorNames`, `executorFactoryNames`,
  `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`,
  `unanalyzableSql`, and `sqlIncludeLocked`. `scope: repository` is also an
  error, because repository scope is a second, wider pass.
- Absent keys are filled from the project: `importSpecifier`,
  `executorFactoryNames` from `factoryNames`, `executorTypeNames` from
  `typeNames`, `trustedSqlTags`, `schemaCatalogPath`, `unanalyzableSql`, and
  `sqlInclude`.

`include`, `exclude`, `options.include`, `options.exclude`, and
`options.sqlInclude` may only narrow. Excludes are added and are not checked.
An empty rule `sqlInclude` means "use the project list", including when the
project omits `sqlInclude`. A non-empty rule `sqlInclude` against an empty
project list is a widen and a config error. Any other non-empty rule pattern is
allowed only when it is proven narrower: it equals a project pattern, or the
project pattern ends in `/**` and the rule pattern starts with that directory
and contains no `..`. `migrations/**/*.sql` does not end in `/**`, so only an
equal pattern is proven against it. An unproven glob is a config error, not a
wider scan. Rule `include` is proven against the project-root-relative
`include`, and the repository-relative form (`root` plus the pattern) must stay
inside the project root. `options.include` and `options.sqlInclude` are compared
after `root` is prefixed onto the project patterns.

A locked empty `sqlInclude` stays empty for rules that would otherwise replace
an empty list with `**/*.sql`. Rules that do not reference a `postgres` project
keep today's defaults, including per-rule `importSpecifier`. `sqlIncludeLocked`
is internal. Do not set it.
