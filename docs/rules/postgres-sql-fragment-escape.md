# `postgres-sql-fragment-escape`

Flags a cross-file write, rebind, or namespace escape of an exported SQL
fragment. The fragment is a module-level immutable `const` whose initializer is
an interpolation-free trusted SQL tag, and whose name is exported. The finding
is the escape site: the call, export, or binding that writes, rebinds, or
passes the fragment outside an `.append(...)` argument.

`postgres-no-executor-adapter` still proves append chains inside one file. This
rule reads prepared TypeScript facts. When the canonical import graph was
built, it uses recorded import targets. It does not re-parse files and it does
not join SQL identifier interpolation. There is no autofix.

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
shared path filters and are not options of this rule. They select both findings
and gaps. A gap for a file outside that filter is omitted, including when an
included file imports the excluded file. A project `postgres`
block can supply `trustedSqlTags`; do not restate them on the rule when it
does.

## Why and when

An imported fragment that is only passed to `.append(...)` looks static in the
file that appends it. Another file can still call `.append` on the fragment,
rebind it, re-export it, or pass the module namespace to `Object.assign`,
`Object.values`, or `Reflect`. A single-file lint cannot see that escape.
Enable this rule next to the adapter lint when exported SQL constants are
shared across files. A file whose prepared facts include an operational error,
a fatal parse error, or an incomplete value-flow is still checked. A fragment
binding or a write that those facts still record is reported, and the same
file is listed on the check result's `incomplete` field. An unreadable file
is not scanned. Its `incomplete` entry names the file, and a recorded write
is not reported on line 1. A sourced `export { name } from` or `export * as name from` whose recorded
offset is missing is listed on `incomplete` when that module exports a
fragment or the import is unresolved. A local `export { name }` or
`export default name` is not that gap. The message names the file and the
export, and says to restore the namespace export fact. A direct `import()` or
`require()` whose argument is not a string literal, or whose literal specifier
was not recorded, is listed on `incomplete` and is not placed by scanning
source text. A recorded literal, including a type-position `import("...")`
and an `import()` in a function body or call argument, is not that gap. The
message names the file and says to restore the literal import fact, and that
load does not become a finding. One such file does not drop findings from files
whose facts are complete. This rule does not change the shared canonical
graph. When value flow is incomplete, the canonical-graph prerequisite stays
failed, and every other graph rule keeps the same readiness and output as a
run that does not configure this rule. An ESLint-only run does not report
these cross-file escapes. Set `trustedSqlTags` here, or this rule reports
nothing.

## Wall time

Measured on 2026-11-20 on one machine. This is a point-in-time comparison,
not a performance budget. The tree is 301 TypeScript files: one tag module
and 300 modules that default-import it, export an interpolation-free tagged
template, and call `.append` on that export. The check config sets
`trustedSqlTags` to that module's default export. Each runner discards one
warmup and keeps the median of the next three wall-clock samples. Times below
are seconds, rounded to the nearest millisecond.

Typed ESLint is ESLint 10.14.0, `@typescript-eslint/parser` 8.72.2, and
TypeScript 6.0.3, with the parser project service enabled. A local rule reads
the program's source-file list once (364 files, including the TypeScript
library) and requests semantic diagnostics for every file, so the run builds
a typed program. The command is `eslint --config eslint.config.mjs src`.

The Rust sample is a release build of
`no-mistakes check --root <tree> --config <tree>/.no-mistakes.yml --format json`
on the same tree after the ESLint config is removed, so the inventory is the
TypeScript sources. That run reports 300 findings and no `incomplete` entries.

| Runner | Warmup | Samples | Median |
| --- | --- | --- | --- |
| Typed ESLint | 0.639 | 0.627, 0.625, 0.644 | 0.627 |
| Rust `check` | 0.028 | 0.027, 0.025, 0.026 | 0.026 |

The Rust median is lower on this tree.

## What it catches

One finding names the exported fragment and the file that declares it. An
unresolved or ambiguous import names the specifier instead of a file. The line
is the escape:

- `.append` on the fragment, or on a member reached through a namespace import,
  a string key, a rename, a default import, or `export *`. A spread or
  multi-argument `.append`, and a method call on a member of the fragment
  such as `.strings.push` or `.values.splice`, are the same kind of write.
- A non-string computed key, `Object.assign`, `Object.values`, `Reflect.get`,
  `Reflect.set`, or any other use of the namespace that is not one member read.
  That use disproves every exported interpolation-free fragment of the module.
  Destructuring the namespace (`const { COLS } = columns`) and value-position
  `typeof columns` are uses of the whole module. `type Columns = typeof columns`
  and `class Row implements columns.COLS` are types and stay silent. An
  assignment in a class `extends` expression, such as
  `columns.COLS.text = extra`, reports that fragment only.
  `import Local = columns.COLS` names that export alone and stays silent until
  a later write or escape.
- `const parts = require(...)`, `import ext = require(...)`, and
  `await import(...)`, including `.then`. A member on that call, such as
  `require("./producer").version`, is not the whole module.
- An `export { local as public }` of an imported fragment, and an exported
  binding whose value is a member of an imported namespace.

A member read that is the sole argument of `.append(...)` is not an escape.
`query.append(fragment)` and `query.append(ns["FRAGMENT"])` stay silent. A `!`
or `as` assertion around that argument is left to `postgres-no-executor-adapter`.
The same-file adapter finding and this rule can both report when one file both
calls the executor and escapes the fragment.

Each import uses the shared resolver's target for that specifier. The rule
does not match a specifier to a neighbor by file stem. When that target is
a file, the file is used even if its name differs from the specifier. When
the target is missing or unresolved, a relative specifier is joined
lexically to the importer, and a specifier can match a file that contains
`declare module "specifier"` when that file is unique. When those steps do
not name one file, or the recorded targets disagree, the use is reported.
That finding names the specifier and does not attribute a local file. A use
of the configured trusted tag itself is not a fragment, so an unresolved or
ambiguous import of that tag stays silent. Any other binding from that
specifier is still reported.

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
