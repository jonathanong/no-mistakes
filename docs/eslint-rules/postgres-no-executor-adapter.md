# `no-mistakes/postgres-no-executor-adapter`

## Why

A callback that only forwards its arguments to a trusted SQL executor hides
that call from static analysis. `run((statement) => read(statement), account)`
builds the statement inside `run` and reaches `read` only through the lambda
parameter, so the executor call is unanalyzable. Passing the function value,
`run(read, account)`, is the same blind spot: analysis does not connect an
executor argument to the statement inside the helper.

Name the executor on the SQL text inside the helper. That shape is judged:

```ts
function run(account: string) {
  return read(sql`SELECT id FROM orders WHERE account_id = ${account}`);
}
```

These call shapes are also judged:

```ts
read(sql);
const exec = read;
exec(sql);
(flag ? read : write)(sql);
tx(sql);
tx.query(sql);
```

`const` is required for an imported-executor alias. A `let` binding and
`(flag ? client.query : read)(sql)` are not judged. Analysis does judge a const
alias of a scoped factory or typed parameter: `const t = tx; t(sql)` and
`const t = tx; t.query(sql)` are offset. This rule does not suggest that alias;
its message names `tx(sql)` or `tx.query(sql)`. An untyped `client.query(sql)`
has no receiver provenance, so it is not a judged offset.

This rule is opt-in. It is not part of `configs.recommended` or `configs.strict`.
It does not ban ordinary object mutation; that is [issue #2219](https://github.com/jonathanong/no-mistakes/issues/2219).

## Disallowed

```ts
import { read, write, type Tx } from "@example/db";

declare function run(exec: (statement: string) => unknown, account: string): unknown;

export function list(account: string, flag: boolean, tx: Tx) {
  run((statement) => read(statement), account);
  run(flag ? read : write, account);
  run(read, account);
  run(tx.query, account);
  run(flag ? read : other, account);
  run((statement) => (flag ? read(statement) : normalize(statement)), account);
  const selected = flag ? read : write;
  run(selected, account);
  const adapter = (statement: string) => read(statement);
  run(adapter, account);
}
```

Passing an executor value is reported and not rewritten. The message names the
executor inside the helper (``read(sql`…`)``, or ``write(sql`…`)``) instead of
passing it. A `const` bound to a conditional of executors also names
``(flag ? read : write)(sql`…`)`` when every branch is an identifier executor.
`run(read, account)` is not a judged rewrite.

A conditional reports when any branch is an executor. The message names only
those branches (``read(sql`…`)`` for a passed value, `read(sql)` inside a
callback), not the unknown branch (`other`, `normalize`, or an untyped
`client.query`) and not ``(flag ? read : other)(sql`…`)``, which would claim that
branch is an executor. Deleting the unknown branch is not a suggested rewrite.

Conditional executor selection written at the call is reported the same way,
still without a rewrite. The message names `read(sql)` or `write(sql)`, and
`(flag ? read : write)(sql)` when every branch is an identifier executor. It
also names `run(read, account)` and says that passing the function is not a
judged rewrite. The same report, still without a rewrite, covers a different
receiver or `this`, an arity or rest change, a side effect, an async or
generator wrapper, and a type assertion. A `const` whose initializer is that
kind of callback is reported too.

## Allowed

```ts
import sql from "sql-template-strings";
import { read, write } from "@example/db";

declare function run(exec: (statement: string) => unknown, account: string): unknown;

export function list(account: string, flag: boolean, client: { query: (sql: string) => unknown }) {
  read("SELECT id FROM orders WHERE account_id = $1");
  const exec = read;
  exec("SELECT id FROM orders WHERE account_id = $1");
  (flag ? read : write)("SELECT id FROM orders WHERE account_id = $1");
  // Not a judged offset: this receiver has no configured type or factory proof.
  client.query("SELECT id FROM orders WHERE account_id = $1");
  function load(account: string) {
    return read(sql`SELECT id FROM orders WHERE account_id = ${account}`);
  }
  // A zero-argument thunk forwards nothing.
  run(() => read());
  // A shadowed name, another module, or a callback that changes the argument
  // is not an unchanged forwarder.
  client.execute((statement: string) => statement.trim());
}
```

Not followed, not safe: the rule does not follow a constructor argument or an
object property. No report for these shapes is not a judgment that the adapter
is safe.

```ts
new Runner((statement: string) => read(statement));
const options = { exec: (statement: string) => read(statement) };
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
- A configured module matches `.query` only when the receiver is a configured
  import, a scoped factory or typed parameter, or a `const` alias of one.
  An untyped `client.query` is not an executor. Without a module, `.query`
  matches every member only when `executorNames` includes `query`.
- Omitting both `importSpecifier` and `executorNames` is a configuration error.
  Set `executorNames: []` to select no executor calls.

## Fix

This rule does not autofix and does not offer a suggestion. Replacing the
adapter with `run(read, account)` is not a judged rewrite: analysis does not
follow an executor passed as a value, so the statement inside `run` stays
unchecked. Name the executor inside the helper instead:

```ts
function run(account: string) {
  return read(sql`SELECT id FROM orders WHERE account_id = ${account}`);
}
```

The message for a passed executor names that shape (``read(sql`…`)``, and
``const exec = read; exec(sql`…`)`` when the executor is an imported identifier).
It does not rewrite the call.

A callback can also be replaced by a direct executor call. Write the SQL as an
argument of the executor (or of a `const` alias of an imported executor):

```ts
read("SELECT id FROM orders WHERE account_id = $1");
const exec = read;
exec("SELECT id FROM orders WHERE account_id = $1");
(flag ? read : write)("SELECT id FROM orders WHERE account_id = $1");
```

`flag ? read : write`, `tx.query(statement)` on a proven receiver, `read.call` / `read.apply`,
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
