# `postgres-require-query-annotation`

Require a leading `/* name */` block comment on executed PostgreSQL SQL so
slow-query logs and `EXPLAIN ANALYZE` can name the statement. Line comments
(`-- name`) do not count. `BEGIN`, `COMMIT`, and `ROLLBACK` are exempt.

The rule consumes prepared embedded-SQL and helper summaries from the request's
shared TypeScript parse. Imported helpers use the request's prepared importer
project catalog and source session. Package-local TypeScript aliases resolve from
the importing file's project; an explicit `--tsconfig` deliberately overrides that
ownership for every importer.

SQL initialized in a `var` declaration stays visible in its enclosing function
or program after a conditional or loop block ends. `let` and `const` stay inside
their lexical block, and nested functions own their bindings. Reassigned or
conflicting SQL initializers remain unresolved rather than selecting one branch.

```yaml
rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      importSpecifier: "@example/db"
      executorNames: [query, read, write]
```

`importSpecifier` has no default. `executorNames` defaults to `query`, `read`, and `write` only when `importSpecifier` is configured.

Counterexample: `query(\`SELECT id FROM posts\`)`.

```ts
import { query } from "@example/db";

export function list() {
  return query(`SELECT id FROM posts ORDER BY id DESC`);
}
```

Fix: put a non-empty block comment at the start of the executed SQL.

```ts
query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

Use `no-mistakes-disable-next-line postgres-require-query-annotation` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule when query logs, slow-query reports, and `EXPLAIN ANALYZE` output
need a stable operation name rather than an opaque SQL string.

## What it catches/requires

Executed PostgreSQL SQL must begin with a non-empty block comment. `BEGIN`,
`COMMIT`, and `ROLLBACK` are exempt; line comments do not satisfy the contract.

## Options and defaults

`include` and `exclude` select source files. `importSpecifier` has no default, and `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

### Executor configuration

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `importSpecifier` to your database module or list `executorNames` explicitly.
Use `executorNames: []` without a module to select no executor calls and retain
SQL-file/native-SQL analysis where supported. See the
[executor migration](../migrations/explicit-postgres-executors.md).

| Option            | Default                                                      | Behavior                                                                            |
| ----------------- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `importSpecifier` | Empty                                                        | Set explicitly to your database module to match its named imports.                  |
| `executorNames`   | Empty without a module; `[query, read, write]` with a module | Without a module, only explicitly listed names match named imports from any module. |
| `executorFactoryNames` | Empty | Named imports (from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty) whose call result bound with `const`, `let`, `using`, or `await using` is an executor inside the declaring block. |
| `executorTypeNames` | Empty | Imported type names (`import type` or inline `type` specifiers) from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty, whose annotated parameters, including optional and inline-destructured ones, are executors inside the declaring function. |
| `trustedSqlTags` | Empty | Named imports of `name` from `module`, or a subpath of `module`, are parameterized SQL tags. A renamed local binding is trusted. A default import is not. A shadowed or rebound local fails closed. The same name from another module, or a sibling prefix such as `@example/dbx`, stays untrusted. |
| `unanalyzableSql` | `report` | Report configured executor arguments whose leading SQL cannot be verified. Set `ignore` explicitly to retain the earlier behavior of skipping opaque arguments. |

### SQL helpers and callbacks

Straight-line same-file and imported helpers can return SQL assembled from strings,
templates, nested SQL builders, and `.append()` calls. Empty fragments are skipped
in composition order. Literal arguments can establish interpolated template text;
unknown text before the first stable prefix remains unanalyzable. A leading
`/* name */` stays valid when a later appended fragment is opaque:

```ts
function ordersSql(select: string) {
  return sql``.append(`SELECT ${select} FROM orders`);
}
write(ordersSql("id")); // Missing annotation.
write(sql`/* orders/list */ `.append(ordersSql("id"))); // Valid.
```

Named and default imports, plus named and star re-exports, can resolve helper
functions. Namespace imports remain unanalyzable. Captured local bindings use the
value available when the helper runs; calling before initialization remains opaque.
Imported helpers share their module bindings during tracing, so an opaque mutator
invalidates a module builder before a later helper reads it.
Existing local `sql` template-tag implementations retain their previous behavior
and can forward annotation prefixes through imported helpers. Unary and sequence
expressions are traversed for nested executor calls without treating those
wrappers as SQL values.
Await an async helper before passing its returned SQL to an executor; a promise passed
without awaiting it remains unanalyzable. Template substitutions are traversed for nested
executor calls. Unknown calls or untrusted template tags that receive a mutable
SQL builder invalidate its previous prefix, including aliases to that builder.

Callback forwarding through a straight-line helper substitutes the statement and
callback arguments at each analyzable callsite. Findings point to the executor
inside the callback. Every callsite must pass; one annotated invocation cannot
hide an unannotated or opaque invocation of the same callback.

```ts
async function runSql(statement, run) { return run(statement); }
runSql(ordersSql("id"), statement => write(statement)); // Executor needs an annotation.
```

Tracing is bounded and conservative. Cycles, reassignment, unsupported control
flow, unresolved imports, arbitrary external calls, and unknown leading fragments
produce an unanalyzable finding by default. Make the leading fragment static,
prepend a named block comment at the caller, suppress an intentional exception,
or explicitly configure `unanalyzableSql: ignore`. This default also applies to
opaque executor arguments that earlier versions silently skipped. Include/exclude
filters select reported files; an imported helper outside that selection can still
be traced through the prepared project facts. Prefix evidence is used only by this
annotation rule and does not make dynamic SQL complete for other PostgreSQL rules.
An incomplete transaction prefix such as `BE` plus unknown text remains
unanalyzable because it could complete an exempt `BEGIN` statement.

`executorFactoryNames` and `executorTypeNames` add scoped executors: `tx` in
`await using tx = await openTransaction()` or `run` in `run: TxExecutor` is scanned
only inside the block or function that declares it, and a same-named identifier
elsewhere is not. Matching is by configuration only, both default to empty, and
neither counts as executor selection: still set `importSpecifier` or `executorNames`.
With `importSpecifier` set, these two options also match imports from its subpaths
(`@example/db/types` for `@example/db`) but not sibling packages such as
`@example/dbx`. A relative import (`../transaction`) matches only when the resolved
file is inside the package `importSpecifier` resolves to; a same-named import outside
that package does not. If the package root cannot be determined, relative imports do
not match. `executorNames` still requires the exact module.

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

## Valid example

```ts
query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

## Counterexample

```ts
query(`SELECT id FROM posts ORDER BY id DESC`);
```

## Fix

Add a short operation identifier as the first block comment inside the SQL
string passed to the configured executor.

## Suppression

Use `no-mistakes-disable-next-line postgres-require-query-annotation` or
`no-mistakes-disable-line`; use the file directive for a deliberately opaque
administrative script.

## Related rules

[`postgres-no-offset`](postgres-no-offset.md) discourages unstable pagination;
[`postgres-lock-ordering`](postgres-lock-ordering.md) protects concurrent row
locks.

Speculative function entrypoints use isolated initialized module state. Actual helper and callback call chains retain shared builder state. Bare `var` redeclarations preserve existing parameters and hoisted functions. Logical and conditional expressions contribute every syntactically possible helper invocation; helper calls in unsupported syntax remain conservative rather than allowing a favorable modeled call to hide unknown arguments. Diamond star re-exports of the same original binding resolve to that binding.

Opaque mutations invalidate previously proven prefixes of mutable builders, including method receivers, spread arguments, nested containers, property writes/deletes, and constructor inputs. Replacing or deleting `String.raw` revokes built-in tag trust; writes to a lexically shadowed `String` leave the global built-in unaffected. Spread arguments remain conservative for positional helper and callback substitution.

Conditional and logical arms use independent mutable state. An unchanged leading
annotation remains provable, while a possible opaque mutation makes the SQL
unanalyzable. Unsupported control flow containing potentially mutating calls
cannot restore an earlier prefix through legacy recovery. A hoisted local `var`
shadows a captured binding, including when it has no initializer.

An initialized `var` redeclaration assigns when its initializer executes, preserving
a parameter value used earlier. Regular helpers own an `arguments` object; arrows
inherit it from their enclosing helper, so opaque mutations through that object
retain the same builder identity. Static numeric and canonical numeric-string
indices select the corresponding argument; dynamic indices remain conservative.
Untrusted local tags can mutate captured
builders even when their own bodies use supported straight-line syntax. They can
also invoke interpolated callbacks, invalidating captured builder prefixes.
