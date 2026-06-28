# `no-mistakes server contracts`

Compare statically extracted client route references with server route handler
query parameter usage.

```sh
no-mistakes server contracts --format json
no-mistakes server contracts /api/search --format md
```

The report lists each extracted route, the query params the backend handler
appears to read, static client references with query strings, and advisory
mismatches where a client sends query params the matched handler does not read.

This check is intentionally conservative. It only reports statically visible
route refs and statically visible query param reads such as `req.query.foo`,
destructuring from `req.query`, Hono `c.req.query("foo")`, and direct
`URLSearchParams(...).get("foo")` reads.

Key options: `--root`, `--tsconfig`, repeatable `--filter`, `--format`,
`--json`, and `--timings`.

Node API: `serverContracts(options)`.
