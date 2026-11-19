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
  load(query: QueryExecutor) {
    return query("SELECT id FROM orders");
  },
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
An optional parameter (`options?: QueryOptions`) or an explicit
`QueryOptions | undefined | null` declares one options object, so a helper such
as `rejectUnresolvable(id, { query })` is accepted, including in a guarded
`if (!row) return rejectUnresolvable(id, { query })`. The accepted property is
the one the checker resolves to a configured executor type on that declared
parameter. An untyped or `any` parameter, a property typed `unknown` or as a
plain function, a union of distinct options types, and a type only inferred
from a default value give no declared contract and keep reporting (an
executor-typed body that calls the property with visible SQL is already
judged by SQL visibility). So does an options object that also escapes in the
helper, for example `stash.push(options)`.
An optional parameter with an inline type (`options?: { query: SessionExecutor }`)
behaves like a required one, including when the body reads `options!.query(...)`.

A shared options interface may declare the executor as optional when the
helper's parameter has a default value. The call site supplies the property
with `{ query }` or `{ query: tx }`, so it is present for that call:

```ts
interface QueryOptions {
  query?: QueryExecutor;
  readOnly?: boolean;
}
async function assertDelivered(id: string, options: QueryOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}
async function rejectUnresolvable(id: string, options: QueryOptions = {}) {
  await assertDelivered(id, options); // passed on unchanged
  assert(false, 404, "not found");
}
rejectUnresolvable(id, { query });
```

When the helper passes its `options` binding on as a direct call argument, the
proof is applied to the callee. The callee must be a single non-overloaded
function declaration with a body, an identifier parameter at that position, and
the same declared options type; the chain may span at most four functions and
must not revisit one. The binding must not be written, spread (`{ ...options }`),
cast, stored (`stash.push(options)`) or mutated (`options.readOnly = true`) in
the helper, and spread arguments, constructors, arrow `const` callees and
destructured callee parameters stay unproven. An optional property needs a
parameter default (`options: QueryOptions = {}`) or an optional parameter
(`options?: QueryOptions`): the call site supplies the property in its literal.
Without either, an optional property, a property set to something other than an
executor (`{ query: (statement) => read(statement) }`), a different key
(`{ label: query }`) and a callee with another options type keep reporting.

### Library executor terminals

The options chain may also end at a configured library executor instead of
`options.query(...)`. The call must name an executor from `executorNames`
imported from `importSpecifier` (`read`, `write`, ...); no name heuristics apply,
and a function that is not configured (`lookup(statement, options)`) keeps reporting.
The options binding is the second argument, passed unchanged or as the first
spread of an object literal followed only by literal scalar overrides:

```ts
async function sendWidget(id: string, options: QueryOptions = {}) {
  await write(sql`UPDATE widgets SET state = 'sent' WHERE id = ${id}`, options);
  await write(sql`UPDATE widgets SET state = 'sent' WHERE id = ${id}`, {
    ...options,
    readOnly: false,
  });
}
sendWidget(id, { query });
```

The binding is judged like a pass-on: writing or storing it, or mutating a
property such as `options.readOnly`, keeps the call reporting. So do
`{ readOnly: false, ...options }`, an override that is not a literal scalar
(`{ ...options, query: other }`), a computed key, and a second spread. The
statement still goes through the SQL proof above in both modes: a trusted-tag
literal with scalar holes. `write(statement, options)` with a received or
dynamic statement keeps reporting.

A single-statement adapter closure that passes its parameter straight through
to such a call is followed when it is an argument of a helper whose executor
parameter is proven, and the helper's own SQL is the proven literal:

```ts
async function selectWidget(id: string, execute: SessionExecutor) {
  return execute(sql`SELECT id FROM widgets WHERE id = ${id}`);
}
async function fromPrimary(id: string, options?: QueryOptions) {
  return selectWidget(id, (statement) => write(statement, { ...options, readOnly: false }));
}
fromPrimary(id, { query });
```

The closure must be non-async with one identifier parameter and either an
expression body or a single `return`. The options argument must be an unwritten
parameter of the enclosing function, optionally spread with literal scalar
overrides. A closure with more statements, one that transforms the statement
(`write(String(statement), options)`), async or generator closures, options
from another source, and a closure that is not passed directly as an argument
keep reporting as before. A closure without an options argument
(`(statement) => read(statement)`) also keeps reporting.

The receiving helper's parameter may also be a plain function type instead of
an executor type, so a helper that takes `run: (statement: Statement) =>
Promise<unknown>` accepts such a closure. The helper must be a local function
without a rest parameter and call that parameter only with the proven literal
SQL; an imported helper, an untyped or destructured parameter, and a parameter
that is written keep reporting.

An options type whose `query` property is declared inside the configured
library (`QueryOptions` and `QueryExecutor` both declared in
`importSpecifier`'s package, or in the file a relative `importSpecifier`
names) counts as the configured executor when the property's type name is in
`executorTypeNames`. A same-named type declared elsewhere does not.

An options type the file imports from `importSpecifier` (including a re-export
or alias of a type another package declares, such as a path-aliased module that
runs `export type { QueryOptions }` on a third-party type) counts too. The
checker follows the aliases to the `query` property, whose type must resolve to
a declaration named in `executorTypeNames`; no other property may be typed as
one. A lookalike declared locally or imported from another module, a `query`
typed as anything else, an unresolved type and a second executor-typed
property keep reporting.

Both reporting modes judge an executor supplied this way (a defaulted optional
property, or a callee reached through a pass-on) by the SQL it receives, the
same proof `forwarding-only` applies: a trusted-tag literal with scalar holes,
or such a statement extended with `.append(<literal>)`. A dynamic fragment
(`statement.append(column)`) keeps reporting.

### Append chains

A helper that forwards `query(sql`...`.append(COLS).append(sql`...`))` is proven
when the chain's root and every appended fragment are trusted `sql` tags, or a
fragment is a constant: a module-level `const` bound to a trusted `sql` tag that
nothing reassigns or mutates. The only uses allowed anywhere in the program are
as the argument of `.append(...)`; `COLS.append(...)`, passing it to another
call, or exporting it as a value that is read otherwise count as mutation. A
constant declared in the helper's file may interpolate scalars. A constant
imported from a sibling module (including `import * as ns` access and renamed
imports) must have no interpolations, and the checker must be available because
the proof reads every file of the program. Without type information only
constants declared in the same file and never exported are proven. Uses are
resolved through re-exports, renames and `default` aliases, so a mutation in
another module through any alias disproves the constant.

```ts
const COLS = sql`id, state`;
async function appendOnly(id: string, query: SessionExecutor) {
  await query(sql`SELECT `.append(COLS).append(sql` FROM widgets WHERE id = ${id}`));
}
```

An appended parameter, `let`, call result, string constant, a constant that is
mutated anywhere (including in another module), a constant whose tag has
interpolations from another module, or a constant used as the chain root
(`COLS.append(...)` mutates it) keeps reporting.

Structural callable lookalikes, computed keys, and objects
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
passing it. That call form is for an executor type that has call signatures. A
configured type with none, such as a pool client passed as `query.client` or
`transaction.client`, is named inside the helper without a call form, or as
``client.query(sql`…`)`` when the type has a `query` method. A `const` bound to
a conditional of executors also names
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

| Option                 | Default                                                    | Purpose                                                                                             |
| ---------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `importSpecifier`      | `""`                                                       | Select the database import boundary.                                                                |
| `executorNames`        | `query`, `read`, `write` with a module; required otherwise | Select named executors.                                                                             |
| `executorFactoryNames` | `[]`                                                       | Select imported executor factories.                                                                 |
| `executorTypeNames`    | `[]`                                                       | Select imported executor types.                                                                     |
| `trustedSqlTags`       | `[]`                                                       | Select SQL tags, including default imports, whose templates expose SQL in `"forwarding-only"` mode. |
| `nonWritingCallees`    | `[]`                                                       | Trust exact imported non-writing callees in `"forwarding-only"` mode.                               |
| `mode`                 | `"all"`                                                    | Use `"forwarding-only"` to narrow reports to forwarders and hidden SQL.                             |

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
- In `"forwarding-only"` mode, a same-file helper may also pass its executor
  parameter on, unchanged, as a plain argument to another same-file helper. The
  callee is proven only when every hop is itself proven, with the same literal
  SQL, hole-safety and executor-use checks: any other use of the parameter (a
  store, a closure return, an escape, a reassignment, a spread or an unresolved
  or overloaded callee) keeps the call reported. Hops may sit in loops and run
  one after another. The check is depth-bounded (eight nested helpers) and a
  cycle between helpers is unproven. A `for...of` const bound to an element of a
  parameter, or to a primitive, is a proven projection or scalar when nothing
  writes it; an interpolation that uses any other iteration value reports. A
  `const summary = await load(query, id)` result, where `load` is a proven
  helper called with a typed executor parameter, is a valid projection receiver
  for a later helper under the same typed field proof.
- Those hop arguments may also be built from the executor's own rows or from a
  proven helper. `rows.map((r) => r.id)` and
  `rows.map((r) => ({ a: 1, b: r.id, c: "x" as const }))` are accepted when
  `rows` is a never-written `const { rows } = await query(...)` of a typed
  executor, the callback is a single arrow expression, and every projected
  field passes the same typed scalar proof as a direct projection. The awaited
  result of a proven helper called with the same executor is accepted whether
  or not it is scalar (for example `number[]`), inline as
  `deliver(await build(ids, query), query)` or through a const. A mapped array
  or result binding that is later mutated or reassigned reports. A callback
  with a block body, a side effect or more than one expression, whole rows
  (`rows.map((r) => r)`), spreads of a row, non-scalar fields, other receivers,
  a result of an unproven helper and dynamic SQL in any hop keep the call
  reported.
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
- With typed ESLint, a stable object parameter can supply a required primitive
  data field as a bind value. The checker must resolve one unambiguous data-field
  declaration and prove a string, number, boolean, bigint, null, or a union
  entirely composed of those primitives. This includes
  named aliases and interfaces, inherited required fields, direct member reads,
  and one-level local or parameter destructuring (including renamed bindings).
  A scalar helper argument such as `input.id` uses the same proof when the
  helper interpolates that value directly. A property interpolation requires
  proof of that named property; a scalar argument alone cannot supply it. Every
  branch of an object union must have the required primitive field. Existing default
  parameter handling still determines whether `undefined` is accepted.
- A `const` local is a valid projection receiver, with the same proof as a
  parameter, when it is initialized from a parameter, an element or member of
  a parameter (`const notice = rows[0]!`), or an element of a row set
  destructured from `await query(...)` where `query` is a parameter typed with
  a configured `executorTypeNames` entry, or a `const` or `await using` bound to
  `await factory()` where `factory` is an imported `executorFactoryNames` entry
  (`const tx = await beginTransaction()`, then `const { rows } = await tx(statement)`).
  That factory binding is an executor root like the typed parameter, as long as
  it is never reassigned, not a `let`, and not passed to another function before
  the projection. A factory missing from the list, a call that is not awaited, and a
  reassigned `let` stay reported. The local's type must supply the
  required primitive field, and neither the local nor its source parameter or
  row set may be written, aliased, iterated, passed to a writing callee, or
  otherwise escape anywhere in the function. A `let`/`var` local, a local
  initialized from `any`, an unknown call, or another function's result, and a
  shadowing declaration remain reported. An assertion between declaration and
  projection (`assert(notice, 404)`) is judged by the assertion proof below.
  The same proof covers a `const` local initialized from another proven `const`
  local (`const first = rows[0]!; const notice = first`), a destructured field of a
  proven receiver (`const { id } = notice`; a later write to `notice.id` cannot change
  the copied `id`), and a field of a `const` object literal read once with no
  spread, computed key, getter, write, or escape (`const notice = { id: "a" }`,
  then `notice.id`). A write to the source before the destructuring, a `let`,
  a computed key, a spread, or a second use of the literal stays reported. The
  proof requires the field to be primitive-typed, which is why a later write
  through the source cannot change the copy.

  An early-exit guard also proves a nullable local: `if (!row) return null`,
  `if (!row) throw …`, `if (!row) return reject(id, query)` and
  `if (row == null) …`, `if (row === undefined) …` and `if (row != null) { … }`
  are plain reads of the local (the local on the left, `null` or `undefined` on
  the right, as the `if` test or an operand of `||` / `&&` in it, such as
  `if (!row || row.locked) return null`), and the checker's narrowed type at the
  projection decides. The projection is accepted only when the guard always
  exits or encloses it, so the local is non-nullable there. A guard that does
  not exit (`if (!row) log()`), a guard on another variable, a guard after the
  projection, a write to the local before it, a reversed comparison
  (`null === row`), a comparison with anything else (`row === other`,
  `row > 0`) and a `!row` that is not an `if` test (`!row ? a : b`) stay
  reported.
  The write, escape, alias or writing-callee check is per projection site: only
  uses between the declaration and that projection, in source order, invalidate
  it. A later `return notice`, a later SQL interpolation, or a later scalar
  projection (`markNotice(notice.id, query)`) leaves earlier projections proved,
  while a later write still invalidates every projection after it. A use inside a
  loop that also contains the projection counts as before it, including when the
  const is declared inside the loop, unless the const is rooted at a binding
  declared inside that same loop (a fresh object each iteration); a write through
  `const row = rows[i]!` to `rows` declared outside the loop still reports. A use
  inside any function closure counts regardless of position; a projection inside
  a closure of the declaring function is checked against every use.
  Parameter receivers (`widget.id` where `widget` is a parameter) follow the same
  per-projection rule: a later use of the parameter does not invalidate an earlier
  projection, while a write, escape or alias that may precede it does. Primitive
  member arguments of one call (`pair(query, widget.id, widget.ownerId)`) do not
  invalidate each other; an object member argument (`stash(widget.nested)`) before
  the projection still does. A scalar read bound to a `const`
  (`const id = row.id`) is not an alias of the row, so later projections such as
  `const ownerId = row.ownerId` stay proved; `const copy = row` still is one.

- An object-literal argument is proved property by property with the same checks as
  a positional argument: `markWithInput(tx, { id: row.id, ownerId: row.ownerId, at })`
  is accepted when the executor is a typed parameter or a configured factory root
  and every property the helper reads is a proven projection or an accepted scalar.
  A shorthand property (`{ id }`) counts only when `id` is itself a proven
  projection binding. These stay reported: a spread (`{ ...row }`), the whole row
  as a value, a computed key, a call on the row or a write to it
  (`row.id.toString()`, `(row.id = x)`), and an object literal stored in a variable
  and mutated or passed on before the call.
- The primitive-type requirement applies to values read from the row. A value
  unrelated to the row is judged exactly as a positional argument is, and a
  property and a positional argument now use one check. Before this change, both
  rejected a non-row `Date` (`markWidget(row.id, at, tx)` with `at: Date`, or
  `{ id: row.id, reviewedAt: at }`). Both now accept a non-row bind value that is a
  primitive or the built-in `Date`: a parameter typed that way (with or without
  a default such as `at = new Date()`), a `const` bound to `new Date(...)`, or an inline
  `new Date(...)` whose arguments are proven scalars. This is sound because a
  `Date` is a bind value, never a SQL fragment, and a value that does not come
  from the row cannot carry stale row data. A `const` Date is accepted only
  when no later use (a setter call, a write, an escape) could write row data into
  it. These stay reported, in both positions: a `Date` or other non-primitive row
  field (`row.at`, `new Date(row.at)`), the row, the rows, or an alias of either,
  an object derived from the row (`{ ...row }`, `Object.assign({}, row)`, a local
  built from row fields and then mutated), a shadowed `Date`, and a non-row object
  or array, whose contents cannot be proved free of SQL fragments. `string | null`
  row fields still prove.
- In `"forwarding-only"` mode, an optional property that an inline
  object-literal argument omits (`put(tx, { id: row.id })` where the callee
  type declares `ownerId?: string`) reads as `undefined` and proves, because
  nothing row-derived reaches it. Default `mode: "all"` is unchanged. Extra
  optional fields the callee never binds into SQL stay silent. An inline
  `new Date()` still proves. These stay reported: a required property, a
  spread, a `__proto__` key, an object that is not written inline at the call,
  `{ id } as Req` when `Req` requires the omitted field, and an omitted field
  fed to `.append(input.note ?? '')`. When the callee binds every field, a row
  `Date`, a row array, a non-row object, and a mutated `Date` still report.
- Two shapes are silent today by design, not by proof: a root whose factory is not
  in `executorFactoryNames` (`declare function openOther(): Promise<SessionExecutor>`)
  and `const pending = beginTransaction(); const tx = await pending`. Neither is
  an executor root the rule recognizes, so the call is never judged. Fixtures pin
  both; list the factory to have it judged.
- Passing that object whole to another call preserves the projection proof only
  when typed ESLint resolves a single stable, visible synchronous function
  declaration whose matching parameter has required primitive data fields and
  only reads them directly or uses one-level destructuring into scalar `const` bindings.
  Its body must consist of scalar expressions, constants, returns, and simple
  `if`/block control flow. Other calls, external property reads, writes, deferred
  code, and loops remain unproven, even when the formal parameter itself is not
  assigned. Other arguments must also be side-effect-free scalar literals,
  unwritten primitive parameters, or bounded scalar `const` initializer chains;
  calls and property/getter reads in those arguments remain unproven.
  This applies before or after the projection, also
  across source imports. A readonly annotation alone is not evidence: mutable
  implementations, ambient declarations without the assertion proof below,
  unknown/`any` callees, captures,
  storage, aliases, and field writes remain unproven. Default/rest/optional
  parameters, overloads, optional/spread calls, methods, and computed access are
  outside this narrow call proof. Calls through captured or reassigned bindings
  do not gain an exemption merely because they appear after a value read.
- With typed ESLint, an assertion signature can preserve an otherwise proven
  object projection:
  `asserts value` or `asserts value is T` must assert the parameter at the exact
  position where the object is passed directly. Ambient declarations are
  trusted as typed contracts. Visible implementations must use only simple
  guards, throws, and bare returns; other calls, writes, and captures are
  unproven. A throw may decorate a fresh built-in error with scalar metadata:
  `throw Object.assign(new Error("assertion failed"), { status })`.
  This requires the unchanged built-in `Object.assign`, `Object`, and `Error`
  in both the assertion module and its caller. Prototype references and
  escapes of these intrinsics remain unproven because inherited setters can
  run during assignment. Metadata must be own, static data properties with
  literal or proven primitive parameter values; getters,
  spreads, computed keys, and object values remain unproven. This writes only
  the fresh error, not the asserted object. General `Object.assign` calls,
  including writes to an assertion parameter, remain unproven.
  Guards can check the parameter itself, such as `if (!value)`, but cannot read
  object fields. A field check such as `typeof value.id !== "string"` remains
  unproven even when TypeScript narrows the object; optional fields also fail
  the ordinary reader proof. A built-in `Error` constructor must remain
  unchanged in both the assertion module and its caller.
  Overloaded assertions qualify only when every call signature asserts the
  same argument position. A visible implementation must still pass the body
  proof, and mixed assertion/boolean overloads remain unproven.
  An assertion about another argument, an ordinary boolean return, `any`, or
  `unknown` does not establish this proof. Passing the same object again in
  another argument remains
  unproven. Other arguments must satisfy the side-effect-free scalar checks
  above. Assertions do not widen the caller-side projection contract: every
  caller union member must still provide the required scalar field.
- `nonWritingCallees` defaults to `[]`. Each entry has `module` and `export`;
  use `export: "default"` for a default import or the exported name for a
  named import. Stable function declarations and `const` function initializers
  are supported. Matching uses the exact import binding, never the local name;
  type-only import or re-export paths cannot prove a runtime callee.
  A shadowed binding or an import from another module does not match.
  This is an explicit trust boundary: configure only functions you trust not
  to mutate, retain, or capture the directly passed object. A configured
  helper may have a known `void` or boolean return type: configuration trusts
  its non-writing contract, but that return type supplies no narrowing.
  Without configuration, a `void` or ordinary boolean signature proves nothing.
  Configuration does not turn `any` or `unknown` into a proven callee.
  Reassigned bindings, spread calls, object captures, unsafe other arguments,
  and writes between the assertion and SQL projection remain unproven.
  Assertion signatures and this option preserve bind-value analysis only in
  `"forwarding-only"` mode; default `mode: "all"` is unchanged.
- This proof applies only to interpolations of configured parameterized trusted
  tags. A field passed as statement text, string concatenation, an interpolated
  raw template, an untrusted tag, a SQL/statement object, `any`, or `unknown`
  remains unproven. Optional or missing fields, getters, computed or nested
  projections, private members, casts standing in for type proof, and mutated or
  escaped objects also remain reported. Loop targets such as `for (input.id of ids)` and
  `for (input.id in ids)` count as writes. A reassigned destructured binding is
  unproven in both local and imported helpers. Without a TypeScript program, the existing
  literal-object and scalar-parameter controls keep their syntax-only proof;
  the new typed-field proof is unavailable. Default `mode: "all"` is unchanged.
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
      trustedSqlTags: [{ module: "sql-template-strings", name: "default" }],
      nonWritingCallees: [{ module: "@example/assertions", export: "default" }],
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

With a configured trusted `sql` tag, required primitive fields are placeholders,
not statement text:

```ts
type Input = { id: string };
function count(query: QueryExecutor, input: Input) {
  const { id } = input;
  return query(sql`SELECT count(*) FROM sample WHERE id = ${id}`);
}
function load(input: Input) {
  return count(read, input); // allowed with typed ESLint
}
```

A typed assertion can narrow optional input before its field becomes a bind
value. This synthetic example uses the configured tag and assertion import:

```ts
import sql from "sql-template-strings";
import assert from "@example/assertions";
import { read, type QueryExecutor } from "@example/db";

type Notice = { id: string };
function count(query: QueryExecutor, id: string) {
  return query(sql`SELECT count(*) FROM sample WHERE id = ${id}`);
}
function load(notice: Notice | undefined) {
  assert(notice);
  return count(read, notice.id);
}
```

Here the assertion export has an `asserts value` signature. At `notice.id`,
the receiver `notice` must narrow from `Notice | undefined` to `Notice`.
The rule then checks that `id` is a required scalar field, with type `string`;
the wider input annotation alone cannot prove that projection. A signature
`asserts value is Notice` can similarly narrow to the required data shape. A cast is not a substitute, and a subsequent
write or escape invalidates the proof. A configured helper returning `void` or
boolean preserves the non-writing contract but supplies no TypeScript narrowing
itself.

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
It does not rewrite the call. A configured executor type with no call signatures
does not use that call. The message names the executor inside the helper and,
when the type has a `query` method, names ``client.query(sql`…`)``. Typed
linting is required to tell those shapes apart. Without a TypeScript program
the message keeps the call form.

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
