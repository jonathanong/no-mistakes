# `no-mistakes/postgres-no-unbounded-query-fanout`

Disallows `Promise.all(<source>.map(<cb>))` when the callback contains a
database executor call and `source` is not statically bounded.

Why: mapping an unbounded collection into parallel executor calls can open
one Postgres query per element. That exhausts the pool, multiplies lock
contention, and turns a linear list into an accidental thundering herd.
Chunked or statically listed work stays within a known concurrency bound.

Example: a static list or chunk helper bounds the fan-out.

```ts
import { query } from "@data-stores/psql";
import { chunkArray } from "../lib/chunk";

const KNOWN_IDS = ["a", "b"];

await Promise.all(KNOWN_IDS.map((id) => query("SELECT 1 FROM t WHERE id = $1", [id])));
await Promise.all(chunkArray(ids, 25).map((id) => query("SELECT 1 FROM t WHERE id = $1", [id])));
```

Counterexample: `Promise.all` fans out an executor over an unbounded map.

```ts
import { query } from "@data-stores/psql";

export function loadAll(ids: string[]) {
  return Promise.all(ids.map((id) => query("SELECT 1 FROM t WHERE id = $1", [id])));
}
```

Fix: bound the collection before mapping. Use an array literal, a
`SCREAMING_CASE` constant, or a configured chunk helper (`chunkArray` by
default), or run the queries sequentially.

A source is statically bounded when it is:

- an `ArrayExpression` (`[a, b].map(...)`)
- a `SCREAMING_CASE` identifier (`KNOWN_IDS.map(...)`)
- a `CallExpression` whose name is in `chunkFunctionNames`

Executor detection uses the same import bindings as
[`postgres-no-manual-transaction`](postgres-no-manual-transaction.md):
`importSpecifier` default `@data-stores/psql`, `executorNames` default
`query` / `read` / `write`, and importing `withTransaction` /
`withTransactionOptions` also binds `query`. Member `.query` calls count as
executors.

Options:

- `importSpecifier` (default `@data-stores/psql`)
- `executorNames` (default `["query", "read", "write"]`)
- `chunkFunctionNames` (default `["chunkArray"]`)

Not in `configs.recommended` or `configs.strict`. Enable it with executor
config for the project.
