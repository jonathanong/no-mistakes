# `postgres-no-unbounded-sql-identifier`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, or `trustedSqlTags` on the rule. `include` and `exclude` may only narrow the project scope.

Flags `process.env`, imported config, request parameters, query result rows, or a local alias of those values interpolated into a SQL relation without a runtime finite-domain validator. A TypeScript union, `satisfies`, or `as const` assertion around external data is not validation. Generated bind values remain allowed.

The request prepares embedded SQL, import edges, and value-flow facts once. This rule borrows those facts and the canonical graph. Recovered switch/validator alternatives that splice literal schemas pass. A call is a validator only when the callee is a proven finite helper (literal returns, typically a switch whose default throws). Identity wrappers, `String()`, and `.toLowerCase()` are not validators. Unknown provenance and HTTP `.query` collisions are not proven violations. There is no autofix.

```yaml
rules:
  - rule: postgres-no-unbounded-sql-identifier
    scope: repository
    options:
      importSpecifier: "@example/db"
```

This example uses repository scope and sets `importSpecifier` on the rule. For
project scope, set it under `projects.<name>.postgres.executors` and select
that project on the rule.

## Why and when

Use this when `config.schema`, `process.env.SCHEMA`, a request parameter, or a database row is concatenated into a relation. Agents otherwise treat the identifier as a closed set.

## What it catches

A configured executor interpolates `process.env.*` (including `process.env["SCHEMA"]`, `import { env } from "node:process"`, and `import { process } from "node:process"`), a local alias of it, a uniquely imported export whose initializer reads `process.env`, a function parameter (`req.params.schema`), or a query-result binding (`result.rows[0].schema`) into a recovered SQL relation. Findings name the producer and the SQL sink and ask for a throwing switch.

Conditional value branches, string concatenation, and local alias chains retain their source provenance. This includes `const e = process.env; e.SCHEMA` and `import * as proc from "node:process"; proc.env.SCHEMA`. A condition that chooses between two literal schemas stays finite; reading env only in that condition is not an unbounded value branch.

Destructured query rows and `for (const row of result.rows)` are checked when their source resolves to a configured SQL executor. This includes array destructuring such as `const [first] = result.rows` followed by `const { schema } = first`, as well as aliased or object-destructured `rows` collections. These projections retain the source row's unbounded provenance; they do not assert a concrete runtime row or field value. Loop destructuring conservatively attributes each bound field to its source row.

Comma expressions use JavaScript's value semantics: only the final operand supplies the interpolated value. An unbounded earlier operand does not make a finite final operand unbounded, while an unbounded final operand remains unbounded. The saved `pass-comma-discarded-env` and `fail-comma-env` fixtures cover both outcomes.

A helper call around those values is reported unless the callee is a proven finite validator. Interpolations come from prepared template-expression facts, not a lexical `${...}` scan of the call span.

## Effort

The default `effort: max` preserves cross-file provenance, helper validation,
and the existing findings. `effort: low` skips JavaScript value evaluation,
call-graph construction, and cross-file provenance for this rule. It reads
the shared parsed program and semantic binding facts instead.

Low accepts a literal SQL string or untagged template directly at a configured
named imported executor, or through one same-file lexical `const` initialized with
that syntax. SQL parsing distinguishes relation interpolation from bound values.
Direct `process.env.KEY` relation inputs whose `process` binding is not shadowed remain proven violations;
a template such as `SELECT * FROM public.users WHERE id = ${id}` stays clear.

Helpers, tags, alias chains, imported SQL values, computed executor selections,
factory/type executors, and CommonJS loaders are unanalyzable at low.
A single local `const run = query` executor alias also remains unanalyzable;
its diagnostic uses the invocation line so a disable-next-line comment above
`run(sql)` suppresses that call.
The existing configured-module `withTransaction` and `withTransactionOptions`
imports also imply executor selection at max. Low conservatively reports their
uses as unknown, including a SQL-free callback, because it does not interpret
callbacks or namespace receiver mutation. Choose max for that form or expose a direct configured executor. Namespace
executor calls and escapes are also unanalyzable; unrelated named methods stay outside
the configured target scope. Rewrite SQL at the direct imported call, keep
relation names literal, and pass external values as bind parameters. For example:

```ts
// Supported low syntax; the environment value occupies a bound-value position.
const statement = `SELECT id FROM public.users WHERE tenant = ${process.env.TENANT}`;
query(statement);

// Unanalyzable at low: expose the statement at the executor instead.
query(buildStatement());
```

Set repository `unanalyzable` or per-rule `options.unanalyzable` to `report`
(default, fail), `advise` (non-failing advisory), or `ignore` (skip unknowns).
`unanalyzableSql` remains an alias. Proven direct environment violations still
fail under all three policies. No partner check is required: unsupported shapes
are explicit unknowns rather than assumed safe. Fatal parse recovery that retains
no configured import or call supplies no SQL proof, matching the max boundary;
parser diagnostics remain in the prepared source facts. Includes, excludes and the
suppression comments below apply to low findings and advisories too.

## Abstains

A runtime-computed environment key has no retained static member name. Use a declared key and a finite validator rather than relying on silence:

```ts
declare const key: string;
query(`SELECT id FROM ${process.env[key]}.users`);
```

New row-projection proofs require a configured query source. An unrelated HTTP response's `.rows` field is not sufficient. Mutable array-destructured bindings and mutable iteration variables (`for (let row ...)` or `for (var row ...)`) are not added to the row provenance map, since a later reassignment can replace the row. Use immutable row bindings and validate the field inside the loop body:

```ts
for (var row of result.rows) {}
row = { schema: "app" };
query(`SELECT id FROM ${row.schema}.users`);
```

For an immutable array-destructured row, one adjacent overwrite is handled precisely: a direct assignment of a primitive scalar to a direct static member, immediately followed by a direct configured query whose only interpolation reads that same member, is treated as finite. The saved `pass-array-row-overwritten-member` fixture demonstrates this counterexample:

```ts
first.schema = "public";
query(`SELECT id FROM ${first.schema}.users`);
```

This proof applies only to adjacent statements. Reads before the write, copied member values, other untouched members, conditional writes, aliases, intervening calls or declarations, object/function assignments, and dynamic writes such as `first.schema = process.env.SCHEMA` retain the row's unbounded finding where its query-source proof applies. In particular, writing one member does not validate the others.

Unresolved imports, invalidated aliases, and cyclic alias chains remain unknown unless another existing source proof applies. Unknown is not a validation guarantee.

## Valid example

```ts
// src/schema.ts
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
// src/query.ts
import { query } from "@example/db";
import { schemaOf } from "./schema";
export function list() {
  return query(`SELECT id FROM ${schemaOf(process.env.SCHEMA ?? "")}.users`);
}
```

A bind-value interpolation (`WHERE tenant = ${process.env.SCHEMA}`), an HTTP `req.query(...)`, or a recovered finite helper stays silent. `${(process.env.UNUSED, "public")}` also stays silent because the final operand is finite.

## Counterexample

```ts
// src/query.ts
import { query } from "@example/db";
export function list() {
  return query(`SELECT id FROM ${process.env.SCHEMA}.users`);
}
```

`const schema = process.env.SCHEMA as "app" | "public"` is the same violation. So are `${identity(process.env.SCHEMA)}`, `${process.env.SCHEMA.toLowerCase()}`, `${process.env["SCHEMA"]}`, `${env.SCHEMA}` after `import { env } from "node:process"`, `${req.params.schema}`, `${result.rows[0].schema}`, and `${("public", process.env.SCHEMA)}`.

## Fix

Map accepted runtime values to literal schemas with a switch whose default throws, then interpolate that helper. Keep values in tagged binds.

## Options and defaults

`importSpecifier` defaults to empty and must be set (or set `executorNames`) to select executor calls when the rule does not inherit a project postgres block. `executorNames` defaults to `query`, `read`, and `write` when a specifier is set. `executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags` default to empty. `include` and `exclude` default to empty (all indexable files).

## Suppression

Use `no-mistakes-disable-next-line postgres-no-unbounded-sql-identifier` or
`no-mistakes-disable-line` at the executor call, or
`no-mistakes-disable-file` for a justified file exception.

## Related rules

[`postgres-no-mutable-sql-registry`](postgres-no-mutable-sql-registry.md) covers cross-file registry member writes.
[`postgres-no-offset`](postgres-no-offset.md) covers pagination clauses.
[`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) covers statement shapes.
