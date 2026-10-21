# `postgres-no-unbounded-sql-identifier`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, or `trustedSqlTags` on the rule. `include` and `exclude` may only narrow the project scope.

Flags `process.env`, imported config, or a local alias of those values interpolated into a SQL relation without a runtime finite-domain validator. A TypeScript union, `satisfies`, or `as const` assertion around external data is not validation. Generated bind values remain allowed.

The request prepares embedded SQL, import edges, and value-flow facts once. This rule borrows those facts and the canonical graph. Recovered switch/validator alternatives that splice literal schemas pass. Unknown provenance and HTTP `.query` collisions are not proven violations. There is no autofix.

```yaml
rules:
  - rule: postgres-no-unbounded-sql-identifier
    scope: repository
    options:
      importSpecifier: "@example/db"
```

## Why and when

Use this when `config.schema` or `process.env.SCHEMA` is concatenated into a relation. Agents otherwise treat the identifier as a closed set.

## What it catches

A configured executor interpolates `process.env.*`, a local alias of it, or a uniquely imported export whose initializer reads `process.env`, into a recovered SQL relation. Findings name the producer and the SQL sink and ask for a throwing switch.

## Valid example

```ts
export function schemaOf(value: string) {
  switch (value) {
    case "app":
      return "app";
    case "public":
      return "public";
    default:
      throw new Error("unsupported schema");
  }
}
```

```ts
import { query } from "@example/db";
import { schemaOf } from "./schema";
export function list() {
  return query(`SELECT id FROM ${schemaOf(process.env.SCHEMA ?? "")}.users`);
}
```

A bind-value interpolation (`WHERE tenant = ${process.env.SCHEMA}`), an HTTP `req.query(...)`, or a recovered finite helper stays silent.

## Counterexample

```ts
import { query } from "@example/db";
export function list() {
  return query(`SELECT id FROM ${process.env.SCHEMA}.users`);
}
```

`const schema = process.env.SCHEMA as "app" | "public"` is the same violation.

## Fix

Map accepted runtime values to literal schemas with a switch whose default throws, then interpolate that helper. Keep values in tagged binds.

## Options and defaults

`importSpecifier` defaults to empty and must be set (or set `executorNames`) to select executor calls. `executorNames` defaults to `query`, `read`, and `write` when a specifier is set. `executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags` default to empty. `include` and `exclude` default to empty (all indexable files).

## Suppression

Use `no-mistakes-disable-next-line postgres-no-unbounded-sql-identifier` or
`no-mistakes-disable-line` at the executor call, or
`no-mistakes-disable-file` for a justified file exception.

## Related rules

[`postgres-no-mutable-sql-registry`](postgres-no-mutable-sql-registry.md) covers cross-file registry member writes.
[`postgres-no-offset`](postgres-no-offset.md) covers pagination clauses.
[`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) covers statement shapes.
