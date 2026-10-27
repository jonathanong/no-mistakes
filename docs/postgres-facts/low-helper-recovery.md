# Bounded low-effort SQL helpers

The low-effort SQL structure rules can recover SQL through a bounded subset of
JavaScript helpers. This is analysis of the helper body, not an exemption for a
typed parameter. It applies to `postgres-no-offset`,
`postgres-no-unbounded-sql-identifier`, `postgres-no-mutable-sql-registry`, and
`postgres-no-transformed-uuid-predicates`, `postgres-explicit-columns`, and
`postgres-required-predicates`. The annotation rule and maximum-effort
analysis keep their existing behavior.

## Supported shapes

Exact local function declarations and resolved named imports/exports can forward
SQL or a configured executor through parameters. Supported helper bodies use local
bindings, literal strings, object fields and destructured/default parameters,
finite conditionals, and fresh `sql-template-strings` builders. For example:

```ts
import sql from 'sql-template-strings';
import { query } from '@example/db';

function suffix(options: { offset: boolean }) {
  return options.offset ? sql` OFFSET 1` : sql``;
}

function run(execute: typeof query, options: { offset: boolean }) {
  const statement = sql`SELECT id FROM app.users`;
  statement.append(suffix(options));
  return execute(statement);
}

run(query, { offset: true }); // OFFSET is reported at its literal in suffix.
```

Every interpolation in an exact imported `sql-template-strings` tag is a bind,
including a string that happens to contain SQL. Appends concatenate recovered
fragments; generated binds are renumbered without rewriting authored text.
Helper arguments retain their physical source locations for findings and
suppression comments. Exact async helpers and `await` are modeled explicitly:
a returned promise is not SQL until an appropriate modeled await unwraps it.

A configured executor passed into a helper is cleared only when the helper's
supported paths prove its uses. Returning or storing an executor in an opaque
value, passing it to an unknown call, or calling it with unknown SQL retains an
unanalyzable diagnostic. A known violation remains a finding even if another
path is unknown, including with `unanalyzable: ignore`.

## Limits and remaining uncertainty

Each site has deterministic limits: eight helper levels, sixteen alternatives,
10,000 work units, and 1 MiB of aggregate constructed SQL. Memoized results pay
the same semantic cost as uncached results. Cycles or exhausted limits remain
uncertain; they are never assumed safe.

This subset does not infer ordinary captured globals, including objects declared
with `const` or `as const`. It does not model generators, loops, switch statements,
callback bodies, mutation of caller-owned builders, rest/spread arguments,
computed object keys, unknown await behavior, or unresolved/ambiguous exports. Imported
helpers must already have facts in the selected low-effort scope; analysis does
not broaden the file selection or parse excluded files as a fallback.

Helpers that receive a SQL object, directly or nested inside an argument, keep
returned SQL read-only because it may alias that input. Such results can be
executed or appended as fragment text, but only calls without incoming SQL objects
can create a fresh builder for further mutation.

For example, `query(buildSqlFromUnknownSource())` remains unanalyzable when that
callee cannot be resolved. Expose a literal at the executor, move the helper into
the supported subset, or select maximum effort. The existing `unanalyzable`
policy controls residual diagnostics: `report` fails, `advise` advises, and
`ignore` skips the residual. None of these settings removes a recovered definite
violation. Existing file and line suppression directives apply at the reported
physical SQL location.
