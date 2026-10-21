# `postgres-no-mutable-sql-registry`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, or `trustedSqlTags` on the rule. `include` and `exclude` may only narrow the project scope.

Flags a member write in one module that mutates an exported SQL identifier registry declared in another module, when a configured executor interpolates that registry into a relation name. A file-local lint cannot prove that path. `no-object-mutation` still owns single-file writes.

The request prepares embedded SQL, import edges, and value-flow mutations once. This rule borrows those facts and the canonical graph. Unknown provenance, HTTP `.query` collisions, and dynamic keys are not proven violations. There is no autofix.

```yaml
projects:
  app:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-mutable-sql-registry
    projects: [app]
```

## Why and when

Use this when one module exports `const tables = {}` and another assigns `tables.users = 'app.users'` before SQL reads it. `const` does not freeze members. Agents otherwise treat the registry as a closed map.

## What it catches

A uniquely imported export receives a known member write in another file, and a configured executor interpolates that binding into a recovered SQL relation (`FROM ${tables.users}`). Findings name the producer, the writer, and the SQL sink.

Writes through a named re-export (`export { tables as registry } from "./tables"`), an `export *` barrel, or a namespace import (`import * as t` then `t.tables.users = ...`) are the same violation as a direct named import. `delete tables.users` and `Object.defineProperty(tables, ...)` are member mutations too.

## Valid example

```ts
export const tables = { users: "app.users" } as const;
```

```ts
import { query } from "@example/db";
import { tables } from "./tables";
export function list() {
  return query(`SELECT id FROM ${tables.users}`);
}
```

A bind-value interpolation (`WHERE name = ${tables.users}`), an HTTP `req.query(...)`, a computed `tables[key] = ...` write, or a write to unrelated state stays silent.

## Counterexample

```ts
export const tables: Record<string, string> = {};
```

```ts
import { tables } from "./tables";
tables.users = "app.users";
```

```ts
import { query } from "@example/db";
import { tables } from "./tables";
export function list() {
  return query(`SELECT id FROM ${tables.users}`);
}
```

`export { tables as registry } from "./tables"` then `registry.users = ...`, `export * from "./tables"` through an index, `import * as t` then `t.tables.users = ...`, `delete tables.users`, and `Object.defineProperty(tables, "users", { value: "app.users" })` are the same class of write.

## Fix

Declare the complete literal registry in one module and import it. Do not assign, delete, or redefine members after export.

```ts
export const tables = { users: "app.users" } as const;
```

## Options and defaults

`importSpecifier` defaults to empty and must be set (or set `executorNames`) to select executor calls when the rule does not inherit a project postgres block. `executorNames` defaults to `query`, `read`, and `write` when a specifier is set. `executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags` default to empty. `include` and `exclude` default to empty (all indexable files).

## Suppression

Use `no-mistakes-disable-next-line postgres-no-mutable-sql-registry` or
`no-mistakes-disable-line` at the executor call, or
`no-mistakes-disable-file` for a justified file exception.

## Related rules

[`postgres-no-unbounded-sql-identifier`](postgres-no-unbounded-sql-identifier.md) covers env/config identifier interpolations.
[`postgres-no-offset`](postgres-no-offset.md) covers pagination clauses.
The opt-in ESLint `no-object-mutation` rule covers single-file member writes.
