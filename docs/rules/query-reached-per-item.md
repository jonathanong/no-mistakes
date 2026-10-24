<!-- cspell:ignore hget -->
# `query-reached-per-item`

Flag per-item calls that transitively reach an explicitly configured effect sink.
This identifies repeated database or cache round trips hidden inside helpers,
including concurrent array callbacks whose bodies contain no literal `await`.
The rule uses prepared TypeScript/JavaScript facts and the canonical call graph.

## Why and when

Enable this rule when repeated single-item queries should be replaced with a
batch helper or folded into a parent query. Parallel promises can still make
one round trip per item; transaction-client calls can execute serially even
when the caller uses `Promise.all`.

## What it catches/requires

Calls inside `for`, `for...of`, `for...in`, `while`, and `do` bodies are per-item.
Callbacks to `map`, `flatMap`, `forEach`, `filter`, `some`, `every`, `reduce`, and
`find` are also per-item. Promise combiners (`all`, `allSettled`, `any`, `race`)
do not make an ordinary fixed array per-item: iteration must supply its calls.
Promises accumulated inside a loop are diagnosed at their per-item calls.

A finding identifies the construct, callee, effect kind, and deterministic
shortest resolved call path to its sink. Transaction sinks are marked separately
and recommend merging statements rather than parallelizing one client.
Unresolved dynamic targets cannot prove sink reachability and are not reported.
Imported receiver mutations remain visible through later object literals,
identifier parameters, and destructured helper arguments or defaults. Replacing
an imported method makes that method unresolved through those receivers; a
method copied before replacement and an untouched method remain proven. Static
nested receiver paths retain the same distinction, while dynamic member writes
invalidate every matching path conservatively. Destructuring acquires a method
at the helper invocation, even when its receiver object was created earlier;
copying the method into that earlier object preserves the original snapshot.

## Options and defaults

The rule is opt-in. Configure effect families under top-level `effects` and
select them with the rule's `options.effects`; no database conventions or sink
names are inferred. Unknown effect kinds are configuration errors.

```yaml
effects:
  postgres:
    targets:
      - { module: '@data-stores/psql', export: read, category: query }
      - { module: '@data-stores/psql', export: write, category: query }
    returnedExecutors:
      - { module: "@vendor/database", export: "beginTransaction" }
    typedExecutors:
      - { module: "@vendor/database", export: "TransactionQuery" }
    returnedMembers:
      - { module: "@vendor/database", export: "writePool.connect", member: "query" }
    transactionFunctions: [tx.query]
    batchFunctions: [readMany]
  valkey:
    targets:
      - { module: '@vendor/cache', export: valkey.hget, category: cache }
rules:
  - rule: query-reached-per-item
    scope: repository
    options:
      effects: [postgres, valkey]
      allow: []
```

| Option    | Default | Behavior                                                                                                      |
| --------- | ------- | ------------------------------------------------------------------------------------------------------------- |
| `effects` | Empty   | Required non-empty list of configured effect family names.                                                    |
| `allow`   | Empty   | Existing callsites to exempt by repository-relative `file`, optional `callee`, and optional one-based `line`. |

Effect `targets` match resolved module/export bindings, including aliased
imports, namespace members, static lexical aliases, and named callbacks. The
rule selects every target in a selected family, regardless of its category.
For example, `import { valkey as cache } from '@vendor/cache'` followed by
`items.map(item => cache.hget(item))` matches `valkey.hget`; a local parameter
named `cache` does not. Calls such as `console.write()` and stream methods do
not match a PostgreSQL target. Transitive helper paths use the same selectors. Visible factory returns and
client parameter/default values, including static object and array parameter patterns, retain their bindings, including qualified
named-client command lists. Unknown or conflicting
client arguments stay unresolved. A configured cursor runner is an effect sink
per invocation; its internal batch size does not exempt an outer loop.

`typedExecutors` selects calls through parameters whose explicit callable type
resolves to the configured imported type export. Explicit named and local type
barrels preserve namespace members such as `Namespace.Query`, including renamed
imports; unrelated local types and different module names remain unselected.
Static indexed property types, including `NonNullable<Options['query']>`, preserve
the selected callable origin. Statically invoked async callbacks preserve returned
executor metadata. A nullish choice between a typed executor and a configured
export is selected only when every alternative belongs to the effect family.
`returnedMembers` selects an exact member call on the receiver returned by the configured imported factory
path. Both use binding provenance, so aliases and local variable names do not
change the match and same-spelled unrelated local methods do not match. These
selectors mark findings as transaction-client-only. They are also available to
the `effects` query, with the configured category. Its callee display uses the
configured type export for typed executors and the factory path plus returned
member for returned members (for example, `writePool.connect().query`).
Namespace-local classes, interfaces, type aliases, enums, and type parameters
shadow imported types, including declarations later in the same lexical body.
A value-only local binding does not shadow a type import. Leaving that scope
restores the imported annotation; matching still requires the configured module
and export identity.

Effect `functions` and `categories` retain the spelling matching documented for
[`effects`](../cli/effects.md), including unrelated bindings with matching names.
Use targets instead of broad spelling lists to avoid those collisions. Adding
targets does not restrict existing lists. `transactionFunctions` names additional sinks
which carry the transaction marker; they require an exact source spelling and
exclude known ordinary local functions for bare names. `returnedExecutors` selects
callable factory results by import binding, regardless of local executor names. `batchFunctions` exempts paths through
configured batch/pipeline builders; another path to a sink remains reportable. Eager query calls in builder
arguments, such as `batch(items.map(item => lookup(item)))`, still report
because they execute before the builder receives their results.
See [effect configuration](../configuration/effects.md).

No type, factory method, or returned member name is inferred from a database
library convention; configure exact exported type and factory/member paths.

Use exact `{file, callee, line}` entries as a reviewed rollout baseline. Omit
`line` to allow that callee throughout one file; omit `callee` to allow the file.
Remove baseline entries as helpers gain batch equivalents.

## Valid example

```yaml
# .no-mistakes.yml
effects:
  postgres:
    targets:
      - { module: '@data-stores/psql', export: read, category: query }
    batchFunctions: [readMany]
rules:
  - rule: query-reached-per-item
    scope: repository
    options:
      effects: [postgres]
```

```ts
// src/users.ts
import { read } from '@data-stores/psql';
async function readMany(ids: readonly string[]) {
  return read(ids);
}
const ids = ['alice', 'bob'];
const users = await readMany(ids);
```

The helper makes one batch call rather than one call per item.

## Counterexample

```yaml
# .no-mistakes.yml
effects:
  postgres:
    targets:
      - { module: '@data-stores/psql', export: read, category: query }
    batchFunctions: [readMany]
rules:
  - rule: query-reached-per-item
    scope: repository
    options:
      effects: [postgres]
```

```ts
// src/users.ts
import { read } from '@data-stores/psql';
async function getUser(id: string) {
  return read(id);
}
const ids = ['alice', 'bob'];
const users = await Promise.all(ids.map((id) => getUser(id)));
```

The report traces the callback's `getUser` call to the configured `read` sink.

## Fix

Use a batch twin of the helper, fold the lookup into the parent query, or add a
reviewed allow entry or suppression when sequential paging, ordered writes, or
rate limits require the per-item behavior. For transaction-client sinks, merge
the statements or batch them on that client.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](README.md#how-analysis-works)); this rule has no unanalyzable inputs because it judges call shapes in parsed source against configured declarations, so `options.unanalyzable` has no effect on it.

## Suppression

Use `no-mistakes-disable-next-line query-reached-per-item` or
`no-mistakes-disable-line query-reached-per-item` at the reported callsite, or
`no-mistakes-disable-file query-reached-per-item` for a justified file exception.

## Related rules

[`forbidden-calls`](forbidden-calls.md) bans configured call paths regardless of
iteration. [`postgres-require-query-annotation`](postgres-require-query-annotation.md)
requires operation names on executed SQL.

CLI: `no-mistakes check --root . --format json`. Node API: the asynchronous
`check({root, config})` and `analyzeProject()` check query expose the same rule.
