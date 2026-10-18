# `no-mistakes/postgres-no-executor-adapter`

## Why

A callback that only forwards its arguments to a trusted SQL executor hides
that call from static analysis. `run((statement) => read(statement), account)`
builds the statement inside `run` and reaches `read` only through the lambda
parameter, so the executor call is unanalyzable. Passing the function value,
`run(read, account)`, is not a repair: analysis does not connect an executor
argument to the statement inside the helper, and the check stays silent.

Call the executor on the SQL text instead. These forms are judged:

```ts
read(sql);
const exec = read;
exec(sql);
(flag ? read : write)(sql);
```

`const` is required for that alias. A `let` binding, a const alias of a scoped
factory or typed parameter, and `(flag ? client.query : read)(sql)` are not
judged. Call `client.query(sql)` or the scoped executor itself (`tx(sql)`).

This rule is opt-in. It is not part of `configs.recommended` or `configs.strict`.
It does not ban ordinary object mutation; that is [issue #2219](https://github.com/jonathanong/no-mistakes/issues/2219).

## Disallowed

```ts
import { read, write } from "@example/db";

declare function run(exec: (statement: string) => unknown, account: string): unknown;

export function list(account: string, flag: boolean) {
  run((statement) => read(statement), account);
  run(flag ? read : write, account);
}
```

Conditional executor selection is reported and not rewritten. The message names
`read(sql)` or `write(sql)`, and `(flag ? read : write)(sql)` when every branch
is an identifier executor. It also names `run(read, account)` and says that
passing the function is not a judged rewrite. The same report, still without a
rewrite, covers a different receiver or `this`, an arity or rest change, a side
effect, an async or generator wrapper, and a type assertion.

## Allowed

```ts
import { read, write } from "@example/db";

declare function run(exec: (statement: string) => unknown, account: string): unknown;

export function list(account: string, flag: boolean, client: { query: (sql: string) => unknown }) {
  read("SELECT id FROM orders WHERE account_id = $1");
  const exec = read;
  exec("SELECT id FROM orders WHERE account_id = $1");
  (flag ? read : write)("SELECT id FROM orders WHERE account_id = $1");
  client.query("SELECT id FROM orders WHERE account_id = $1");
  // Not an adapter. Also not analyzed: the statement inside `run` stays hidden.
  run(read, account);
  // A shadowed name, another module, or a callback that changes the argument
  // is not an unchanged forwarder.
  client.execute((statement: string) => statement.trim());
}
```

## Options

- `importSpecifier` identifies the database module and defaults to empty.
  Configure your database module explicitly. Named executor imports must come
  from that exact module. A subpath such as `@example/db/read` does not match
  `@example/db` for `executorNames`.
- `executorNames` lists checked executor names and defaults to
  `["query", "read", "write"]` only with a configured module; otherwise it is
  empty. A renamed import matches the imported name and the direct form uses
  the local name.
- `executorFactoryNames` lists imports whose call result bound with `const`,
  `let`, `using`, or `await using` is an executor inside the declaring block. It
  defaults to `[]`. `var` is not a scoped executor.
- `executorTypeNames` lists imported type names whose annotated parameters
  (including optional and inline-destructured ones) are executors inside the
  declaring function. It defaults to `[]`.
- `trustedSqlTags` defaults to `[]`. Each entry has `module` and `name`.
  It is accepted so a shared executor configuration validates. This rule does
  not read SQL text.
- Imports for the two scoped options above match `importSpecifier` and its
  subpaths (`@example/db/types` for `@example/db`), not sibling packages such as
  `@example/dbx`. A relative import matches only when it resolves into that
  package. If the package root cannot be determined, relative imports do not match.
- A configured module keeps `.query` member matching even when `executorNames`
  omits `query`. Without a module, `.query` matches only when `executorNames`
  includes `query`.
- Omitting both `importSpecifier` and `executorNames` is a configuration error.
  Set `executorNames: []` to select no executor calls.

## Fix

This rule does not autofix. Replacing the adapter with `run(read, account)`
drops the unanalyzable `read(statement)` call and leaves the statement inside
`run` unchecked. Write the SQL as an argument of the executor (or of a `const`
alias of an imported executor):

```ts
read("SELECT id FROM orders WHERE account_id = $1");
const exec = read;
exec("SELECT id FROM orders WHERE account_id = $1");
(flag ? read : write)("SELECT id FROM orders WHERE account_id = $1");
```

`flag ? read : write`, `client.query(statement)`, `read.call` / `read.apply`,
extra or missing arguments, rest or spread, defaulted parameters, `await`,
`async`, generators, type assertions, and extra statements are still reported.
The message names the direct call and says why that adapter cannot be rewritten.

A callback that does not forward the selector is left alone. That includes a
shadowed executor name, an unrelated object, a different arity that never
passes the parameter, and a transforming call such as `read(normalize(statement))`.

## Suppression

ESLint directives and `no-mistakes` directives both work. A directive for a
different rule does not. Text inside a string is not a directive.

```ts
// eslint-disable-next-line no-mistakes/postgres-no-executor-adapter -- compatibility adapter
run((statement) => read(statement), account);

// no-mistakes-disable-next-line postgres-no-executor-adapter -- reviewed forwarder
run((statement) => read(statement), account);
```

`no-mistakes-disable-line` applies to the same line as the adapter, and
`no-mistakes-disable-file` applies to the whole file when it is the first
comment.

## Related rules

- [`postgres-cursor-call-contract`](postgres-cursor-call-contract.md) requires
  direct, attributable cursor calls.
- [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) keeps
  transaction lifecycle in the owning helper.
- [`postgres-no-unbounded-query-fanout`](postgres-no-unbounded-query-fanout.md)
  bounds concurrent mapped queries.
- General object mutation is tracked separately in
  [issue #2219](https://github.com/jonathanong/no-mistakes/issues/2219). This
  rule does not report ordinary property writes.
