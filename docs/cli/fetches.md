# `no-mistakes fetches`

Map Next.js App Router routes to statically analyzable fetch API calls.

```sh
no-mistakes fetches --root web --format json
no-mistakes fetches /users web/app/users/page.tsx --format md
```

Use this to understand page-to-API coupling and route ownership without running
Next.js. Dynamic fetch URLs or methods are intentionally reported only when the
static analyzer can prove the shape.

Key options: `--root`, `--config`, `--format`, `--json`, and optional route or
file targets.

Reports routes for every configured `type: nextjs` project (or a single
inferred app when none is configured), not just one — each app's rewrites
only expand routes for that app's own route tree. An app whose resolved route
root directory does not exist on disk fails the whole run; an ambiguous
`type: nextjs` project whose root cannot be inferred also fails (see
[Multiple frontend apps](../configuration/tests.md#multiple-frontend-apps)).

Node API: `fetches(options)`.
