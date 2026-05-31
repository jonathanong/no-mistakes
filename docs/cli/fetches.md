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

Node API: `fetches(options)`.
