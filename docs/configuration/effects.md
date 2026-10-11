# Effects

Top-level `effects` declares named families of effect functions or constructors.
The [`effects` query](../cli/effects.md) reports configured occurrences reachable
from an entry file; [`query-reached-per-item`](../rules/query-reached-per-item.md)
uses the same families to detect repeated calls through helpers.

```yaml
effects:
  postgres:
    functions: [read, write]
    categories:
      query: [query]
    targets:
      - { module: "@vendor/database", export: "query", category: query }
    transactionFunctions: [transaction.query]
    batchFunctions: [readMany, pipeline]
```

All lists default to empty. `categories` maps category labels to sink spellings;
`functions` declares uncategorized sinks. Matching uses the exact source spelling
or a member call's terminal name, following the existing effects query. Aliased
imports are not inferred from the configured original name.

`targets` adds binding-aware matching for the `effects` query. Each selector
requires a nonempty `module` import specifier and an `export` member path
(`query`, `default`, or `client.query`); `category` is optional. Unknown selector
fields are rejected. Named/default imports, namespace calls, and static lexical
aliases resolve to the same selector; shadowed local bindings are excluded.
Module specifiers match exactly for repository callable imports, so a different
repository barrel needs its own selector. Explicit re-exports of external
modules resolve to that external module/export selector. Categories filter these
selectors in the same way as categorized spellings. A family may contain only
targets.

String lists retain their existing behavior and take precedence when the same
occurrence also matches a target. Targets configure the effects query; the
per-item rule continues to use its existing spelling lists.

`transactionFunctions` marks additional sinks for per-item rule diagnostics.
`batchFunctions` exempts paths through explicitly configured batch or pipeline
builders for that family. These two fields configure the per-item rule; they do
not change the existing effects query's output or category semantics.

The per-item rule is enabled separately and must select configured family names
with `options.effects`. Nothing is scanned as a database effect by default.
Use `options.allow` for a reviewed baseline of existing per-item callsites, and
normal rule suppression directives for documented exceptions.
