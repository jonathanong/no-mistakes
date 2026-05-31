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
