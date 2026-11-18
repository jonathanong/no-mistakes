# `no-mistakes/postgres-no-executor-adapter`

## Why

A callback that only forwards its arguments to a trusted SQL executor hides
that call from static analysis. `run((statement) => read(statement), account)`
builds the statement inside `run` and reaches `read` only through the lambda
parameter, so the executor call is unanalyzable. Passing the function value,
`run(read, account)`, has the same blind spot when the helper has no configured
executor parameter type.

Name the executor on the SQL text inside the helper. That shape is judged:

```ts
function run(account: string) {
  return read(sql`SELECT id FROM orders WHERE account_id = ${account}`);
}
```

A direct configured executor call already exposes its SQL when the first
argument is an inline string, template, or tagged template. In both modes, a
fresh options object in a later argument may carry a proven executor reference
as a session selector:

```ts
async function update(query: QueryExecutor, id: string) {
  await write(sql`UPDATE orders SET archived = true WHERE id = ${id}`, { query });
  await write("UPDATE orders SET archived = true WHERE id = $1", [id], { query });
}
```

Configured imports, stable aliases, namespace members, and proven executor
receivers use the usual executor resolution. Receiver type proofs require typed
linting. This allowance applies to the executor reference in that fresh object;
forwarding callbacks elsewhere in the same call still report. It does not
extend to helper calls, nested or spread options, computed option keys, optional
calls, or constructors.

The SQL must be visible in the first argument. `write({ query })`,
`write(statement, { query })`, and `write({ query }, "SELECT 1")` still report.
SQL visibility here uses the same inline SQL forms as the SQL checks; it does
not prove bind-only interpolation safety or replace those checks.

A helper with a proven configured executor parameter is also judged. With
`executorTypeNames: ["QueryExecutor"]`, passing the executor directly is
allowed:

```ts
import { read, type QueryExecutor } from "@example/db";
function load(query: QueryExecutor) {
  return query("SELECT id FROM orders");
}
load(read);
const loadOptional = (query?: QueryExecutor) => query?.("SELECT id FROM orders");
loadOptional(read);
function loadObject({ query }: { query: QueryExecutor }) {
  return query("SELECT id FROM orders");
}
loadObject({ query: read });
```

Same-file function declarations and `const` arrow/function expressions use the
parameter's declared imported type. Optional and defaulted parameters and inline
destructured properties without property defaults are supported. A shadowed type
name cannot prove the configured type. Forwarding callbacks still report,
including `load((statement) => read(statement))`.

A same-file `const` object literal can expose a method, arrow, or function
property. The receiver must stay private and closed: every reference must be a
direct call of a known own function property, using a static property name.
Literal string or template keys are supported. Aliasing, exporting, returning,
or passing the receiver elsewhere, property writes, spreads, getters, duplicate
keys, optional calls, `this`/`super` access, and direct `eval` keep the target
unproven. Transparent TypeScript casts do not bypass the direct `eval` guard. A `let` receiver or a dynamically selected property also keeps reporting.

```ts
const helpers = {
  load(query: QueryExecutor) { return query("SELECT id FROM orders"); },
};
helpers.load(read);
helpers["load"](read);
```

With typed ESLint, a direct namespace import can also name a stable exported
function or `const` function initializer, such as `helpers.load(read)` after
`import * as helpers from "./helpers"`. The implementation and configured
executor parameter type must be available in the TypeScript program. A namespace
alias or an imported mutable object does not establish this proof. These helper
targets do not add executor names or imports to the configured discovery scope.

A parameter may also use a union when **every** member is a configured executor
type imported from the configured module or `typeof` a configured executor
named import from that module. `QueryExecutor | typeof read` and `typeof read`
are proven; `QueryExecutor | ((statement: unknown) => unknown)` is not. This
works for same-file helpers and, with typed linting, imported helpers. Namespace
imports also qualify: `db.QueryExecutor | typeof db.read` for
`import * as db from "@example/db"`. Parentheses around members or the entire
union do not change proof. Every member must still match: mixed callable types,
same-spelled exports from another module, shadowed namespace bindings, and
nested namespace paths remain unproven. A type-only namespace import can prove
a configured type but cannot prove a runtime `typeof` executor. SQL
checks recognize these proven parameters inside the helper. A `typeof` executor
import follows the same exact `importSpecifier` boundary as `executorNames`;
configured type imports may also come from its subpaths. With explicit
`executorNames` and no module, named executor imports from any module qualify.

Imported helpers require typed linting: configure `@typescript-eslint/parser`
with `parserOptions.project` or `projectService` so `parserServices.program`
is available. The declared parameter must reference a configured type imported
from the configured database module, and its signature must belong to the
imported function declaration or `const` function initializer. The exporting
source must have no direct `eval` call, and no assignments, destructuring writes,
updates, or loop assignments to that helper binding. Writes to a shadowed local
binding do not invalidate the exported helper. Without that information, the
call reports.
The implementation body must also be available in the TypeScript program.
Ambient function declarations and ambient callable variables keep reporting:
their signatures describe a contract but do not expose SQL for analysis.
Contextual function types such as `const run: Helper = query => query(sql)`
do not annotate the implementation parameter and still report.
Structural compatibility, inference, unknown callees, mutable helpers,
overloads, rest parameters, and calls containing spread arguments do not
establish proof.
For example, `load(...[read])` reports; use the explicit argument `load(read)`.
With typed linting, a fresh options object can also target a declared object
parameter. The type checker must resolve the named property to a required field
declared with a configured imported executor type. Interfaces, type aliases,
inherited fields, and named destructured options types are supported. A field
may use a local alias of the configured type; an intersection must resolve to
one unambiguous required field declaration:

```ts
interface QueryOptions {
  query: QueryExecutor;
  limit?: number;
}
function dispatch(options: QueryOptions) {
  return options.query("SELECT id FROM orders");
}
dispatch({ query: read });
const query = read;
dispatch({ query });
```

This proof requires typed linting for both local and imported options types.
Parser fallback programs with module resolution disabled cannot prove named or
imported options contracts, so those arguments keep reporting.
The shared executor discovery also follows these declared properties inside
the helper, including a destructured alias such as
`function load({ query: execute }: QueryOptions)`. SQL checks inspect the
executor calls there. A property may have another name, such as `execute`,
when its declared type proves the same configured executor identity.
Executor uses inside the helper must be direct calls or stable local `const`
aliases that are called directly. Judged `.call`, literal-array `.apply`, and
optional calls are also supported. Returning or storing the options object or
executor value, forwarding it to opaque code, or replacing the property keeps
the incoming call reported. Aliasing or destructuring the whole options object
inside the body is also unproven; call the declared property directly instead.
Optional fields, structural callable lookalikes, computed keys, and objects
containing a spread still report. Nested containers such as
`dispatch({ nested: { query: read } })` and forwarding callbacks remain
unproven. Use an explicit required executor property or name the executor inside
the helper.

An executor can also be a key in a typed built-in collection. A stable receiver
typed as the standard-library `Map` or `WeakMap` accepts argument zero of
`.get`, `.set`, `.has`, and `.delete`; `Set` and `WeakSet` accept argument zero
of `.has` and `.delete`. These operations do not invoke the key:

```ts
const locks = new WeakMap<QueryExecutor, object>();
locks.set(read, {});
locks.get(read);
locks.delete(read);
```

This requires typed linting and an unchanged built-in method. The receiver must
come from a proven built-in allocation or a stable alias of one. A parameter
annotation alone cannot prove that a custom implementation was not supplied.
The key type need not name an executor: native `new Map()` and
`Map<any, any>` allocations can also prove a key operation.
Unknown receivers, same-named custom collections, overridden methods, escaped
receivers, and spread or computed calls still report. Storing an executor as a
value, such as
`locks.set(otherKey, read)` or `set.add(read)`, still reports because it can be
retrieved and called later. Put data in the value slot instead.
Assignments to aliases and storage inside another collection can expose a
receiver to method replacement, so key uses of those receivers keep reporting.

These call shapes are also judged:

```ts
read(sql);
const exec = read;
exec(sql);
(flag ? read : write)(sql);
tx(sql);
tx.query(sql);
db.read(sql);
const { query } = tx;
query(sql);
read?.(sql);
read["call"](null, sql);
read["apply"](null, [sql]);
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
declare class Runner {
  constructor(exec: (statement: string) => unknown);
}

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
  function forward(statement: string) {
    return read(statement);
  }
  run(forward, account);
  new Runner(read);
  new Runner((statement: string) => read(statement));
  run({ exec: read }, account);
  run([read], account);
  run(read.bind(null), account);
  run(read ?? write, account);
  run(flag && read, account);
  run(flag === 1 && read, account);
  run(typeof flag && read, account);
}
```

Passing an executor value to an unproven helper is reported and not rewritten.
The message names the executor inside the helper (``read(sql`…`)``, or ``write(sql`…`)``) instead of
passing it. A `const` bound to a conditional of executors also names
``(flag ? read : write)(sql`…`)`` when every branch is an identifier executor.
`run(read, account)` is not a judged rewrite when `run` has no proven configured
executor parameter. The same passed-value report
covers a fresh object property or array element, a `new` argument, and
`read.bind(null)`. The rejected quote keeps the property or element around
that executor (`run({ exec: read }, account)`, `run([read], account)`).

A `function` declaration is followed when that binding is never assigned and
no direct `eval` can reach it. A TypeScript overload signature is not a second
definition and does not block that rewrite. The message matches the `const`
arrow form. `read ?? write` and `read || write` name only `read`: a proven
executor is never nullish or falsy, so the right operand is not the value.
`read && write` names only `write`. `flag && read`, `flag === 1 && read`,
`typeof flag && read`, and `flag ?? read` name only `read`, not the whole
expression. A left operand that might be nullish reports the same way unless
it is provably never nullish: `void 0 ?? read`, `client.flag ?? read`,
``tag`q` ?? read``, `this ?? read`, and `new.target ?? read` (inside a plain
function) name only `read`. `typeof` and `new` are always truthy, so
`typeof flag && read` and `new Runner() && read` always pass `read`. A
comparison, `!`, unary `+` or `-`, or a sequence can be truthy or falsy, so
those left operands are followed too (`!flag && read`, `+account.length && read`,
`(account, flag) && read`).

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

Not followed, not safe: a missing report is not a judgment that the adapter is
safe. The rule does not follow a stored object or array (properties can
change), a property read, a spread element, a getter or setter, or a private
method. It does not follow `read.bind` passed without calling it, or `.bind`
on a callback. `read?.bind(null)` and `read.bind?.(null)` are that bind when
the chain runs. A string or expression-free template key is the same member
(`db["read"]`, ``db[`read`]``, `read["call"]`, ``read[`apply`]``). A computed
identifier key (`db[name]`, `read[name]`) is not a static name. `.bind`
with arguments after `this` (`read.bind(null, account)`) is not followed:
those arguments are prepended, and `read(sql`…`)` would drop them. A function
declaration that is assigned, or that shares its scope with direct `eval`, is
not followed. A function expression's inner name is not followed; the `const`
binding is the one this rule judges. A left operand that cannot select the
executor stays silent. `typeof flag || read` and `new Runner() || read` are
always the left side. `typeof flag ?? read`, `flag === 1 ?? read`,
`` `x${flag}` ?? read ``, `[] ?? read`, `new Runner() ?? read`, and
`!flag ?? read` stay silent. `typeof`, an untagged template, `new`, an array,
object, or function literal, a comparison or other binary expression,
`!` or unary `+` `-` `~`, `delete`, an update expression, and dynamic
`import()` are never nullish. `void` is always `undefined`, so
it is not in that set. Literal operands are folded:
`0 && read`, `1 || read`, and `0 ?? read` are silent because that executor is
not the value. An untyped `client.query` under a configured module is not
named, including when it appears in these shapes.

```ts
const options = { exec: (statement: string) => read(statement) };
run(options, account);
run(options.exec, account);
const fns = [read];
run(fns[0], account);
run(((statement: string) => read(statement)).bind(null), account);
run(read.bind(null, account), account);
function adapter(statement: string) {
  return read(statement);
}
adapter = other;
run(adapter, account);
run(typeof flag || read, account);
run(new Runner() || read, account);
run(flag === 1 ?? read, account);
run(`x${flag}` ?? read, account);
run([] ?? read, account);
run(new Runner() ?? read, account);
run(!flag ?? read, account);
run(flag++ ?? read, account);
run(import("m") ?? read, account);
const obj = { k: true };
run(delete obj.k ?? read, account);
run(0 ?? read, account);
```

## Options

```js
{
  "no-mistakes/postgres-no-executor-adapter": [
    "error",
    { importSpecifier: "@example/db" },
  ],
}
```

| Option                 | Default                                                    | Purpose                                                                       |
| ---------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `importSpecifier`      | `""`                                                       | Select the database import boundary.                                          |
| `executorNames`        | `query`, `read`, `write` with a module; required otherwise | Select named executors.                                                       |
| `executorFactoryNames` | `[]`                                                       | Select imported executor factories.                                           |
| `executorTypeNames`    | `[]`                                                       | Select imported executor types.                                               |
| `trustedSqlTags`       | `[]`                                                       | Select SQL tags, including default imports, whose templates expose SQL in `"forwarding-only"` mode. |
| `mode`                 | `"all"`                                                    | Use `"forwarding-only"` to narrow reports to forwarders and hidden SQL.       |

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
  declaring function. It also proves supported helper argument slots described
  above, including proven required options-object fields with typed linting.
  It defaults to `[]`.
- `mode` is `"all"` by default. Set `"forwarding-only"` to report forwarding
  callbacks and blocked or unproven SQL paths while allowing a stable same-file
  helper to receive an executor directly or in a fresh options object when
  every use of that parameter calls it with a literal SQL string or static
  template. With typed linting, the same check applies to imported helpers.
  A helper that calls the executor with a received or composed SQL fragment
  still reports. This option changes only this rule.
- `trustedSqlTags` defaults to `[]`. Each entry has `module` and `name`.
  In `"forwarding-only"` mode, a direct tagged template is visible SQL only
  when its tag resolves to that configured import and the template starts
  with non-empty static text. When a template interpolation comes from a helper
  parameter, the rule follows stable local bindings to that parameter and checks
  the value supplied at the call. A received SQL fragment or an opaque value
  keeps the call reported; a proven primitive scalar, including a literal or
  stable scalar binding, stays a placeholder. Later writes make a binding
  unproven. A possibly `undefined` value is unproven when it can activate a
  SQL-fragment default parameter. Other modes accept this option for shared
  executor configurations without using it.
  Configure `name: "default"` for `import sql from "sql-template-strings"`;
  named imports use their exported name. For a static object argument, the
  same scalar or fragment check follows a destructured property such as
  `function search({ where }, query)` or `const { where } = options` inside
  the helper. Computed keys, spreads, and unknown objects remain unproven.
- Imports for the two scoped options above match `importSpecifier` and its
  subpaths (`@example/db/types` for `@example/db`), not sibling packages such as
  `@example/dbx`. A relative import matches only when it resolves into that
  package. If the package root cannot be determined, relative imports do not match.
- A configured module matches `.query` only when the receiver is a configured
  import, a scoped factory or typed parameter, or a `const` alias of one.
  An untyped `client.query` is not an executor. Without a module, `.query`
  matches every member only when `executorNames` includes `query`.
  Unbounded-fanout and nonliteral-registry-key checks use this same receiver
  boundary; a class method merely named `query` is not enough.
  `postgres-no-manual-transaction` does not: when `executorNames` includes
  `query`, it matches every statically named `.query(...)` call, including an
  unproven class or client method.
  `import * as db` matches `db.read` when `read` is an executor name and the
  module is `importSpecifier` (a subpath does not match). `const { read } = db`
  is that export. `const { query } = tx` is `tx.query` when `tx` is a proven
  receiver. A computed identifier key, a property default, `let` or `var`, and
  a default import are not those executors. A member named `call` on the
  namespace is that executor, not `Function.prototype.call`, when `call` is the
  configured name.
- Omitting both `importSpecifier` and `executorNames` is a configuration error.
  Set `executorNames: []` to select no executor calls. The option schema sets
  `additionalProperties: false`, so ESLint rejects an `extraExecutorNames` key
  as a configuration error.

For example, with `mode: "forwarding-only"`:

```js
{
  "no-mistakes/postgres-no-executor-adapter": [
    "error",
    {
      importSpecifier: "@example/db",
      executorTypeNames: ["QueryExecutor"],
      mode: "forwarding-only",
    },
  ],
}
```

```ts
function load(query) {
  return query("SELECT id FROM orders");
}
load(read); // allowed: the executor call names visible SQL
load((statement) => read(statement)); // reported: forwarding callback

function search(fragment, query) {
  return query(fragment);
}
search(sql`SELECT id FROM orders`, read); // reported: SQL is hidden from the executor call

function filter(where, query) {
  return query(sql`SELECT id FROM orders WHERE ${where}`);
}
filter(sql`active = true`, read); // reported: the helper receives a SQL fragment
filter(true, read); // allowed: the helper receives a scalar placeholder

function filterOptions({ where }, query) {
  return query(sql`SELECT id FROM orders WHERE ${where}`);
}
filterOptions({ where: sql`active = true` }, read); // reported: fragment in a destructured property
filterOptions({ where: true }, read); // allowed: scalar in a destructured property
```

The mode also accepts a fresh object such as `load({ query: read })` when a
stable local helper calls that property with visible SQL. It does not infer
contracts from names, callable structural types, mutable helpers, or imported
implementations without typed linting. Under Oxlint, including `--type-aware`,
JavaScript plugins receive no TypeScript program. Imported callees therefore
cannot be proven and keep reporting. Use ESLint with
`@typescript-eslint/parser` and `parserOptions.projectService` for this rule.
Interpolated raw templates and arbitrary tagged templates are unproven in this
mode. Configure a trusted SQL tag when placeholders are needed. A conditional
argument with only configured executor branches is allowed when the local or
typed imported helper names visible SQL; an unknown branch still reports.

## Fix

This rule does not autofix and does not offer a suggestion. Replacing the
adapter with `run(read, account)` requires a proven configured executor
parameter on `run`; otherwise the statement inside `run` stays unchecked. Name
the executor inside the helper instead:

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
