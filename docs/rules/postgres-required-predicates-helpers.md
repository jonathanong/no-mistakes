# postgres-required-predicates — Executor SQL built by helpers

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

### Executor SQL built by helpers

Executor SQL assembled from helpers is checked, including helpers imported
from another module and helpers that interpolate caller-supplied string
literals. With `unanalyzableSql: ignore`, an unrecoverable call is skipped
silently, so recovery is what makes such a read visible.

```ts
// visibility-sql.mts
export function ownerVisibleSql(parentCol: string, ownerCol: string): string {
  return `(${parentCol} IS NULL OR EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol}))`;
}
// caller.mts
const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
query.append(ownerVisibleSql("l.parent_id", "l.account_id"));
await read(query);
```

The literals are spliced into the helper's template, so the `view_active_plans`
read above is reported (on the helper's line, in the helper's file). Imports
resolve through relative specifiers with explicit extensions, `tsconfig`
`paths` aliases, pnpm workspace package names and barrel re-exports (up to 8
hops).

Exported string constants are recovered through the same imports and barrels.
For `import * as names from "./tables.mts"`, `names.TABLE` splices an exported
literal into an untagged template or `.append` when no local binding shadows
`names`.
`names.default` also selects a default-exported literal. Computed, optional,
method, type-only, and non-const reads stay unanalyzable. Namespace recovery
accepts only string constants: a tagged builder fragment such as `names.FRAGMENT`
stays unanalyzable even in `.append`, because the exported object can mutate
after its declaration. Named and default imports of tagged builder fragments
also stay unanalyzable, including through barrels and `.append`: another
importer can mutate the object. Named imports of primitive string constants
still recover. Use an exported string literal instead of a shared builder when
the fragment must be recoverable without cross-module mutation analysis.

Helpers with control flow are checked too. A helper that returns early, branches
on its arguments, calls other helpers (up to 8 deep), appends to a
`SQLStatement` parameter, builds identifiers from literal object properties or
unrolls a loop over an array literal produces every alternative statement
(at most 16), and each alternative is checked:

```ts
// list-query.mts
export function appendListQuery(query: SQLStatement, options: { all: boolean; limit: number }) {
  if (options.all) {
    query.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans z WHERE z.tier = 'a')`);
    return;
  }
  query.append(sql` LIMIT ${options.limit}`);
}
// caller.mts
const query = sql`SELECT p.id FROM view_active_plans p WHERE p.tier = 'a'`;
appendListQuery(query, { all, limit });
await read(query);
```

Both statements are checked: the caller's own read and the helper's `z` read
are reported. An executor template over a builder's text, ``read(`/* n */ WITH
${b.text}`, b.values)``, and a fluent statement chain, `query.append(a).append(b)`,
are recovered the same way.

A non-literal argument in an identifier position, a helper parameter used
outside a `${...}` interpolation, a loop other than a bounded unroll, more than
16 alternatives, helpers more than 8 deep, a helper cycle, a caller's builder
given to a parameter not annotated `SQLStatement` or read back by the helper, an
unresolved or ambiguous import, a re-export cycle, CommonJS and dynamic
`import()` keep the call `Dynamic`; `unanalyzableSql: fail` then reports it. The
full rules and caps are in
[PostgreSQL fact sources](../postgres-facts.md#imported-and-literal-identifier-helpers).

See [mutable SQL builder recovery](../postgres-builder-mutations.md) for
conservative alias and escape behavior.

[Back to the rule](postgres-required-predicates.md)
