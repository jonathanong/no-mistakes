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
shared across files. Operational errors, incomplete value flow, unreadable
source, and missing prepared facts can add `incomplete` entries within the
runtime dependency closure of a file that reaches a fragment or uses a
configured SQL tag. A tagged template inside a function counts when its tag
resolves to the configured module-level import. A tag import alone does not
establish relevance. Usable prepared facts can still establish a finding and
keep that file in scope for a gap.

The closure follows imports from relevant files to their dependencies. A
file with no usable facts can enter that closure only if a relevant file
depends on it, directly or transitively. For example, a file that fails to
parse may import and mutate a fragment without producing an `incomplete`
entry when no relevant file depends on it.

When value flow is incomplete, graph-dependent sibling rules keep the
canonical graph prerequisite and their own diagnostics. An ESLint-only setup
omits this cross-file check. An empty `trustedSqlTags` list produces no
findings and adds no graph or catalog demand.

A sourced `export { name } from` or `export * as name from` with a missing
recorded offset produces an `incomplete` entry when the target exports a
fragment. A direct `import()` or `require()` with a computed argument or
missing literal specifier fact produces one specifier gap when a binding or
call in the loading file reaches a proven exported SQL fragment. Dependency
coverage gaps also recognize configured tagged queries. A readable source
is required to attach a finding to its line. Use a literal specifier to make
the target analyzable.

Literal loads through a `const` initialized by `createRequire(...)` from
`node:module` or `module` use the recorded call offset. For example,
`load("./columns.js")` can identify the module even when the loader is named
`load`. The rule also uses these facts for type-position imports and imports in
function bodies and call arguments.

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

Literal `require.resolve()` calls return paths and do not seed a module namespace.
The extractor records their kind at the call offset, including named loaders.
Comment and quoted-text lookalikes do not create module-load gaps. A missing
static specifier fact includes a no-substitution template literal, matching
the extractor's literal forms.

## What it catches

One finding names the exported fragment and the file that declares it. The
reported line identifies the write, rebind, or escape.

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

Literal loads through a `const` initialized by `createRequire(...)` are recorded
by the shared import visitor, including calls inside functions. The factory must
come from `node:module` or `module`. A non-literal argument remains unproven.

A member read that is the sole argument of `.append(...)` is not an escape.
`query.append(fragment)` and `query.append(ns["FRAGMENT"])` stay silent. A `!`
or `as` assertion around that argument is left to `postgres-no-executor-adapter`.
The same-file adapter finding and this rule can both report when one file both
calls the executor and escapes the fragment.

The rule uses the request's prepared import targets and TypeScript resolver
scope. Recorded project-file targets are followed exactly, including path
aliases and workspace packages whose filenames differ from their specifiers.
This also covers path aliases and workspace packages when the canonical graph
is unavailable. A unique ambient declaration can identify an external module,
including declarations with whitespace between their tokens. The rule
does not match an import to a neighboring file by file stem. External packages
are not read. Silence at that analysis boundary does not establish that an
unread package contains no SQL fragments.

A failed or ambiguous local code import can add one `incomplete` entry per
file when the file otherwise reaches a proven exported SQL fragment. The
entry identifies the unresolved specifier. Built-in Node modules, external
packages (including external trusted-tag modules and their subpaths), and
resolved non-code files are outside this search. Missing paths and aliases
with a recognized asset extension such as `.json` or `.css` stay silent.
Missing relative and absolute code imports (including Windows drive paths)
and missing code targets of configured path aliases can add an entry.
A `#` specifier can also add an entry when it matches an `imports` mapping in
the importing file's indexed workspace package and its local code target is
missing. Unmapped `#` specifiers and other unknown bare imports stay silent.
A use of the configured trusted tag itself is not a fragment; other
bindings from the same project specifier still follow the ordinary fragment
checks. `check` exits 1 when the report contains an incomplete entry, even if
the report has zero findings.

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
