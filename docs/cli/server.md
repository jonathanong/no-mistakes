# `no-mistakes server`

Analyze Express, Hono, and Koa server route graphs. Configured Django,
Flask, FastAPI, Go `net/http` / Chi / Gin / Echo / Fiber, Rails `routes.rb`,
Laravel, and Symfony attribute/YAML routes are projected into the same
`server routes|edges|related` report from the language `RouteRef` facts.

| Leaf command | Purpose |
| --- | --- |
| [`server routes`](server-routes.md) | List extracted HTTP routes. |
| [`server edges`](server-edges.md) | Print server route dependency edges. |
| [`server related`](server-related.md) | Print files related through route edges. |
| [`server contracts`](server-contracts.md) | Compare static client query params with route handler usage. |

Shared options: `--root`, `--tsconfig`, repeatable `--filter`, `--depth`,
`--format`, `--json`, and `--timings`.
