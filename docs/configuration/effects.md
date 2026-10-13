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
    returnedExecutors:
      - { module: "@vendor/database", export: "beginTransaction", category: query }
    transactionFunctions: [transaction.query]
    batchFunctions: [readMany, pipeline]
```

All lists default to empty. `categories` maps category labels to sink spellings;
`functions` declares uncategorized sinks. These lists match the exact source spelling
or a member call's terminal name, following the existing effects query. Aliased
imports are not inferred from the configured original name.

`targets` adds binding-aware matching for the `effects` query and the per-item
rule. Each selector
requires a nonempty `module` import specifier and an `export` member path
(`query`, `default`, or `client.query`); `category` is optional. Unknown selector
fields are rejected. Named/default imports, namespace calls, and static lexical
aliases resolve to the same selector; shadowed local bindings are excluded.
Repository selectors retain their direct import-specifier boundary for imported
calls; local references to the selected canonical callable also match. Explicit re-exports of external
modules resolve to that external module/export selector. Categories filter these
selectors in the effects query in the same way as categorized spellings. The
per-item rule selects whole families, including targets in every category. A
family may contain only targets.

String lists retain their existing behavior and take precedence when the same
occurrence also matches a target. Use targets instead of broad lists such as
`functions: [read, write, query]` when unrelated methods or local bindings share
those names. Adding targets alongside those lists does not narrow the lists.

Returned functions and destructured factory exports resolve through their visible
factory bodies, including awaited initializers and explicit re-export chains.
Receiver provenance follows client defaults and supplied arguments through helpers.
A shared helper receiver resolves only when its statically known callers agree;
unknown, conflicting, or reassigned values stay unresolved.

`returnedExecutors` selects imported factories whose returned callable is an
effect sink. It uses the same `{module, export, category?}` selectors as `targets`,
and supports aliases, awaited results, and static wrappers without listing local
executor names. Use it when the factory implementation is external or opaque:

```ts
import { beginTransaction as begin } from '@vendor/database';
const arbitraryLocalName = await begin();
await arbitraryLocalName('SELECT 1'); // selected returned executor
```

The factory call itself is not selected by `returnedExecutors`; add a normal
`targets` entry if starting a transaction is also an effect. Returned executors
carry the transaction marker in per-item diagnostics. No transaction factory
names are inferred from a module name or a PostgreSQL convention.

`transactionFunctions` marks additional sinks for per-item rule diagnostics.
It matches exact source spellings, without terminal-name fallback. Bare names
cannot relabel a known ordinary local function as a transaction executor;
use an import target or returned-executor selector for binding-aware selection.
`batchFunctions` exempts paths through explicitly configured batch or pipeline
builders for that family. These two fields configure the per-item rule; they do
not change the existing effects query's output or category semantics.

The per-item rule is enabled separately and must select configured family names
with `options.effects`. Nothing is scanned as a database effect by default.
Use `options.allow` for a reviewed baseline of existing per-item callsites, and
normal rule suppression directives for documented exceptions.
