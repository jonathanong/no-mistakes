# postgres-no-offset — Unanalyzable SQL

### Unanalyzable SQL

**Behavior change:** this rule previously dropped dynamic executor SQL unless
its recovered text already proved an `OFFSET`. With the default
`unanalyzableSql: fail`, a dynamic executor call without complete alternatives is reported with target
`unanalyzable` at the executor call when OFFSET pagination cannot be ruled out:

- no SQL text was recovered (an opaque `query(sql)` argument or a
  `query(cond ? a : b)` choice whose arms cannot all be recovered), or
- the recovered text contains `SELECT`, including a CTE or query inside DML,
  or its leading statement is unknown or incomplete, and it has no OFFSET of
  its own. An interpolation or opaque appended tail could add `OFFSET`.

Recovered keyword checks ignore quoted text, nested PostgreSQL block comments,
and keyword text within identifiers, including `$` and Unicode continuations.

Switch case labels can also change scalar guards before a later case or default
executes. Assignments, updates, destructuring, and directly invoked closure writes
in a label make branching executor SQL inside that switch unanalyzable. The
default `fail` reports the executor line; `ignore` skips that uncertain call.
Literal SQL inside the switch still produces ordinary `offset` findings, and
callbacks that are merely created or passed are not executed by this heuristic.
Literal function or arrow closures invoked via `.call(...)` or `.apply(...)`
also follow this boundary, including TypeScript wrappers and literal computed
property names. Arguments are evaluated before the invoked body. Referencing
`.call` without invoking it, using `.bind(...)`, or passing a callback keeps
that callback body inert.
Creating an unused generator iterator also leaves its body unexecuted, so body
writes alone do not make the query unanalyzable, including literal `.call(...)`
and `.apply(...)` invocations. Arguments and parameter defaults still execute.
Advancing or consuming an iterator (for example, `.next()`,
`for...of`, spread, tagged template interpolation, or `yield*`), storing it, or
passing it to an unknown consumer retains conservative body checks, including
parameter and destructuring defaults.
Stored iterators may remain unanalyzable even when their body is never advanced;
the heuristic does not trace iterator aliases.
An earlier body may be skipped by direct entry or executed by fallthrough, so
its SQL assignments cannot supply a later case's complete statement. Bindings
it changes become opaque for later cases. The default `fail` reports those
calls as `unanalyzable`; `ignore` skips them. Fresh literal and constant SQL
inside a case still receive ordinary checks.

```ts
let gate = false;
switch (2) {
  case ((gate = true), 1):
    break;
  case 2:
    query(gate ? "SELECT 1 OFFSET 1" : "SELECT 1"); // unanalyzable
}

let unchanged = false;
switch (2) {
  case (function* () {
    unchanged = true;
  })():
    break;
  case 2:
    query(unchanged ? "SELECT 1 OFFSET 1" : "SELECT 1"); // passes
}
```

Dynamic DML without a recovered query is not reported, and a
dynamic call whose recovered text already contains `OFFSET` keeps only its
ordinary `offset` finding. See the
[migration note](../migrations/postgres-unanalyzable-sql.md).

```ts
import { query } from "@example/db";

// Counterexample: both calls are reported as unanalyzable by default.
export function run(sql: string) {
  return query(sql);
}

export function sorted(column: string) {
  return query(`SELECT id FROM posts ORDER BY ${column}`);
}
```

Fix: pass a SQL literal or a configured trusted tagged template, pass values
such as page sizes as `$n` binds instead of untagged `${...}` interpolations,
keep pagination clauses in the recovered SQL, suppress the executor line with
`no-mistakes-disable-next-line postgres-no-offset`, or set
`unanalyzableSql: ignore` to restore the earlier skip.

See [mutable SQL builder recovery](../postgres-builder-mutations.md) for
conservative alias and escape behavior.

[Back to the rule](postgres-no-offset.md)
