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

## Options and defaults

The rule is opt-in. Configure effect families under top-level `effects` and
select them with the rule's `options.effects`; no database conventions or sink
names are inferred. Unknown effect kinds are configuration errors.

```yaml
effects:
  postgres:
    functions: [read, write]
    transactionFunctions: [tx.query]
    batchFunctions: [readMany]
rules:
  - rule: query-reached-per-item
    scope: repository
    options:
      effects: [postgres]
      allow: []
```

| Option    | Default | Behavior                                                                                                      |
| --------- | ------- | ------------------------------------------------------------------------------------------------------------- |
| `effects` | Empty   | Required non-empty list of configured effect family names.                                                    |
| `allow`   | Empty   | Existing callsites to exempt by repository-relative `file`, optional `callee`, and optional one-based `line`. |

Effect `functions` and `categories` reuse the spelling matching documented for
[`effects`](../cli/effects.md). `transactionFunctions` names additional sinks
which carry the transaction marker. `batchFunctions` exempts paths through
configured batch/pipeline builders; another path to a sink remains reportable. Eager query calls in builder
arguments, such as `batch(items.map(item => lookup(item)))`, still report
because they execute before the builder receives their results.
See [effect configuration](../configuration/effects.md).

Use exact `{file, callee, line}` entries as a reviewed rollout baseline. Omit
`line` to allow that callee throughout one file; omit `callee` to allow the file.
Remove baseline entries as helpers gain batch equivalents.

## Valid example

```ts
const users = await readMany(ids);
```

A configured batch helper does not produce a per-item sink diagnostic for its
own internal batching path.

## Counterexample

```ts
async function getUser(id: string) {
  return read(id);
}
const users = await Promise.all(ids.map((id) => getUser(id)));
```

The report traces the callback's `getUser` call to `read`.

## Fix

Use a batch twin of the helper, fold the lookup into the parent query, or add a
reviewed allow entry or suppression when sequential paging, ordered writes, or
rate limits require the per-item behavior. For transaction-client sinks, merge
the statements or batch them on that client.

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
