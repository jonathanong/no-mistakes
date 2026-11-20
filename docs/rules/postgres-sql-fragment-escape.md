# `postgres-sql-fragment-escape`

Flags a cross-file write, rebind, or namespace escape of an exported SQL
fragment. The fragment is a module-level immutable `const` whose initializer is
an interpolation-free trusted SQL tag, and whose name is exported. The finding
is the escape site: the call, export, or binding that writes, rebinds, or
passes the fragment outside an `.append(...)` argument.

`postgres-no-executor-adapter` still proves append chains inside one file. This
rule borrows the prepared inventory, TypeScript facts, and canonical import
graph. It does not re-parse files and it does not join SQL identifier
interpolation. There is no autofix.

```yaml
rules:
  - rule: postgres-sql-fragment-escape
    scope: repository
    options:
      trustedSqlTags:
        - module: "@example/sql"
          name: default
```

`trustedSqlTags` is the only rule option. The default is an empty list, and an
empty list reports nothing. Each entry needs a non-empty `module` and `name`.
`name: default` matches a default import of that module, including
`import sql from "@example/sql"`. A bare name matches a named import. `module`
matches that specifier or a subpath under it. `include` and `exclude` are the
shared path filters and are not options of this rule. A project `postgres`
block can supply `trustedSqlTags`; do not restate them on the rule when it
does.

## Why and when

An imported fragment that is only passed to `.append(...)` looks static in the
file that appends it. Another file can still call `.append` on the fragment,
rebind it, re-export it, or pass the module namespace to `Object.assign`,
`Object.values`, or `Reflect`. A single-file lint cannot see that escape.
Enable this rule next to the adapter lint when exported SQL constants are
shared across files.

## What it catches

One finding names the exported fragment and the file that declares it. The line
is the escape:

- `.append` on the fragment, or on a member reached through a namespace import,
  a string key, a rename, a default import, or `export *`.
- A non-string computed key, `Object.assign`, `Object.values`, `Reflect.get`,
  `Reflect.set`, or any other use of the namespace that is not one member read.
  That use disproves every exported interpolation-free fragment of the module.
- `const parts = require(...)`, `import ext = require(...)`, and
  `await import(...)`, including `.then`.
- An `export { local as public }` of an imported fragment, and an exported
  binding whose value is a member of an imported namespace.

A member read that is the sole argument of `.append(...)` is not an escape.
`query.append(fragment)` and `query.append(ns["FRAGMENT"])` stay silent. A `!`
or `as` assertion around that argument is left to `postgres-no-executor-adapter`.
The same-file adapter finding and this rule can both report when one file both
calls the executor and escapes the fragment.

The rule abstains when the graph cannot name one defining module. A relative
specifier with no import edge is joined lexically to the importer. A
non-relative specifier can match a file that contains `declare module "specifier"`
when that file is unique. A re-export whose specifier still does not resolve
matches a fragment only when exactly one exported fragment has that name.

## Valid example

```ts
// widget-columns.ts
import sql from "@example/sql";

export const widgetColumns = sql`id, state`;
```

```ts
// list-widgets.ts
import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { widgetColumns } from "./widget-columns";

export async function listWidgets(query: SessionExecutor, id: string) {
  await query(
    sql`SELECT `.append(widgetColumns).append(sql` FROM widgets WHERE id = ${id}`),
  );
}
```

No other file writes `widgetColumns`, rebinds it, or escapes its module.

## Counterexample

```ts
// widget-columns.ts
import sql from "@example/sql";

export const widgetColumns = sql`id, state`;
```

```ts
// extend-member.ts
import { widgetColumns } from "./widget-columns";

export function extend(extra: unknown) {
  widgetColumns.append(extra);
}
```

```ts
// extend-namespace.ts
import * as columns from "./widget-columns";

export function extend(extra: unknown) {
  Object.assign(columns, { extra });
}
```

The first escape is `widgetColumns.append`. The second disproves every exported
interpolation-free fragment of `./widget-columns`.

## Fix

Keep the fragment in the file that appends it, or stop writing, rebinding, and
re-exporting it. Pass a fresh tagged template to `.append` instead of mutating
the shared constant. Do not pass the module namespace to `Object.assign`,
`Object.values`, or `Reflect`.

## Suppression

```ts
// no-mistakes-disable-next-line postgres-sql-fragment-escape
widgetColumns.append(extra);
```

`no-mistakes-disable-file postgres-sql-fragment-escape` suppresses the whole
file. Suppress the escape site, not the consumer that only appends the fragment.

## Related rules

[`postgres-no-executor-adapter`](../eslint-rules/postgres-no-executor-adapter.md)
proves same-file append chains and reports `!` and `as` assertions.
[`postgres-no-mutable-sql-registry`](postgres-no-mutable-sql-registry.md)
reports member writes that reach a SQL identifier interpolation. This rule does
not replace either check.
