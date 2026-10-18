# Query annotation options and helpers

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
| `unanalyzableSql` | `report` | Report unverified leading SQL. `ignore` skips it; `advise` emits a non-failing residual advisory with reason, known prefix, and helper lines when available. |

### SQL helpers and callbacks

Straight-line same-file and imported helpers can return SQL assembled from strings,
templates, nested SQL builders, and `.append()` calls. Empty fragments are skipped
in composition order. Literal arguments can establish interpolated template text;
unknown text before the first stable prefix remains unanalyzable. A same-file helper can also append static
fragments to a typed `SQLStatement` parameter and return it when the caller's
builder is statically recoverable. Its leading annotation is preserved through
that composition; opaque inputs and dynamic appends remain unanalyzable. A leading
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
Calling a method on an executor's returned value preserves the annotation of
the exact executor call. Chained `catch`, `then`, and `finally` callbacks can
still contain separate executor calls that require their own annotations.

Callback forwarding through a straight-line helper substitutes the statement and
callback arguments at each analyzable callsite. Findings point to the executor
inside the callback. Every callsite must pass; one annotated invocation cannot
hide an unannotated or opaque invocation of the same callback.

```ts
async function runSql(statement, run) { return run(statement); }
runSql(ordersSql("id"), statement => write(statement)); // Executor needs an annotation.
```

Tracing is bounded and conservative. Cycles, unrecoverable reassignment, unsupported control
flow, unresolved imports, arbitrary external calls, and unknown leading fragments
produce an unanalyzable finding by default. Make the leading fragment static,
prepend a named block comment at the caller, suppress an intentional exception,
or explicitly configure `unanalyzableSql: ignore`. This default also applies to
opaque executor arguments that earlier versions silently skipped. Include/exclude
filters select reported files; an imported helper outside that selection can still
be traced through the prepared project facts. Prefix evidence is used only by this
annotation rule and does not make opaque dynamic SQL complete for other PostgreSQL rules.
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
A configured module enables `.query` only on a receiver proven to come from
that module, a configured factory or type, or an immutable executor alias.
Explicit unscoped `executorNames: [query]` matches `.query` broadly. A
configured module also recognizes its transaction helpers. Native SQL and
recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).
