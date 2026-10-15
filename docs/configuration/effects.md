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
    typedExecutors:
      - { module: "@vendor/database", export: "TransactionQuery", category: query }
    returnedMembers:
      - { module: "@vendor/database", export: "writePool.connect", member: "query", category: query }
    transactionFunctions: [transaction.query]
    batchFunctions: [readMany, pipeline]
```

All lists default to empty. `categories` maps category labels to sink spellings;
`functions` declares uncategorized sinks. These lists match the exact source spelling
or a member call's terminal name, following the existing effects query. Aliased
imports are not inferred from a configured bare function name. Qualified
client-member spellings, such as `sessionValkeyClient.scan`, also follow proven
receiver bindings through aliases, parameters, and defaults.

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
Receiver provenance follows client defaults and supplied arguments through helpers, including static object and array parameter patterns with nested bindings and defaults.
A shared helper receiver resolves only when its statically known callers agree;
unknown, conflicting, or reassigned values stay unresolved. Imported clients
created by opaque factories retain their exported receiver identity.

Synchronous inline arrows, function expressions, and returned helpers retain their enclosing
invocation context when resolving supplied receiver arguments and defaults. A
later receiver write in that same context does not invalidate an earlier direct
helper call, including an immediately invoked returned helper imported directly
or through a re-export. Supplier read timing and the helper's file-qualified
callable ownership are tracked separately. Calls after a write and captured closures with uncertain execution
order remain unresolved; synthetic reachability edges do not add a second
unknown argument set to a proven invocation. Returned helpers require every
known factory invocation to call its result immediately; a stored result or
opaque escape keeps receiver arguments unresolved. Async and generator helpers
remain conservative because their receiver reads may occur after suspension.
For callable bodies, an initializer after a suffix that completion analysis
proves unreachable does not establish a captured receiver. A conditional return
keeps an initializer known when another path reaches it; if every branch
returns, later initializers remain unresolved.


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

`typedExecutors` uses the same exact `{module, export, category?}` selector shape
to select calls through a parameter whose explicit callable type comes from an
imported type export. Named and namespace type imports, aliases, and explicit
type re-exports preserve that type provenance. Namespace member suffixes (for example,
`Namespace.Query`) survive explicit named and local type barrels, including aliases.
Local spelling and a matching terminal type name alone do not select a call. These calls carry the transaction
marker in `query-reached-per-item` findings.

`returnedMembers` selects a member call on a receiver returned by a configured
imported factory. Its selector has `{module, export, member, category?}`: `export`
is the exact imported factory path, and `member` is the exact static member path
on its returned receiver. For example, `writePool.connect` with `member: query`
selects calls through the receiver returned by that factory, regardless of local
variable names. Add one entry for each configured factory. This does not match
an unrelated local object with a same-named method. The effects query reports
the callee as the configured factory path followed by `().` and the member
(for example, `writePool.connect().query`) with its configured category.
`typedExecutors` callees display the configured type export, such as
`TransactionQuery`. Per-item findings mark selected returned members as
transaction effects. No type, factory, or member names are inferred.

`transactionFunctions` marks additional sinks for per-item rule diagnostics.
It matches exact source spellings, without terminal-name fallback. Bare names
cannot relabel a known ordinary local function as a transaction executor;
use an import target or returned-executor selector for binding-aware selection.
`batchFunctions` exempts paths through explicitly configured batch or pipeline
builders for that family. These two fields configure the per-item rule; they do
not change the existing effects query's output or category semantics.

`typedExecutors` and `returnedMembers` are opt-in binding selectors shared by
the effects query and the per-item rule. They default to empty and preserve
category filtering in `no-mistakes effects --category`. Existing `targets` and
`returnedExecutors` behavior is unchanged.

The per-item rule is enabled separately and must select configured family names
with `options.effects`. Nothing is scanned as a database effect by default.
Use `options.allow` for a reviewed baseline of existing per-item callsites, and
normal rule suppression directives for documented exceptions.
