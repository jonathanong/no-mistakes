# `no-mistakes effects`

Report every transitive call site of a configured set of effect functions or
constructors that is reachable from an entry file through the import graph.

```sh
no-mistakes effects valkey --entry app/server.ts --root . --format json
```

Use this to map the side effects a server entry pulls in: cache clients,
pub/sub factories, entity-cache getters, invalidators, rate limiters, queue
clients, and so on. Reachability follows runtime import edges (static imports,
dynamic imports, and `require`); type-only imports are ignored. Each reachable
file is parsed once and matching calls — including `new Foo()` constructors and
`obj.method()` calls — are reported with file path, line, category, the
enclosing function (`caller`), and the import depth from the entry.

The function names per `<kind>` are entirely config-driven; nothing is
hardcoded. Configure them under `effects.<kind>` in `.no-mistakes.yml`:

```yaml
effects:
  valkey:
    categories:
      cache: [ValkeyCache, getEntityCache]
      pubsub: [createPublisher, createSubscriber]
      invalidation: [invalidate]
      queue: [GlideMQ]
    targets:
      - module: "@vendor/cache"
        export: "ValkeyCache"
        category: cache
```

Key options: `--entry` (required), `--category` (repeatable, restricts to those
categories), `--depth`, `--tsconfig`, `--config`, `--format`, and `--json`.

Output shape:

```json
{
  "kind": "valkey",
  "entry": "app/server.ts",
  "callSites": [
    { "file": "lib/cache.ts", "line": 4, "callee": "ValkeyCache", "category": "cache", "caller": "makeCache", "depth": 1 }
  ],
  "byCategory": { "cache": 2, "pubsub": 1 }
}
```

`functions` and `categories` retain spelling-based matching: an aliased import
called under another name is not matched, and a same-named local binding can
match. Use `targets` for binding-aware matching by exact import `module` and
`export` path. These selectors follow named/default imports, namespace members,
and statically resolvable lexical aliases, while excluding shadowed bindings.
`category` is optional; selectors without one are uncategorized. A target-only
family is valid. The report's `callee` is the configured export path.

Module selectors match the import specifier exactly for repository callables;
a different repository barrel needs its own selector. Explicit re-exports of
external modules resolve to the external module/export selector. Dynamic calls
and invalidated aliases can remain unresolved.
For an ordinary single-identity call, when both a spelling and a target select one occurrence, the spelling match
keeps its existing category and the occurrence appears once. Reachability
follows import edges from the entry, so a file that `filesystem.skipDirectories`
excludes from discovery can still be reported if it is imported from the entry.
An unknown `<kind>` or a missing entry file is an error.

Node API: `effects(options)`.

Binding-aware selectors retain visible factory returns and client parameter/default
provenance. Configure `effects.<kind>.returnedExecutors` for opaque imported
factories whose returned callable performs the effect. These selectors report
executor invocations, independent of local names, rather than factory creation.
`effects.<kind>.typedExecutors` selects calls through parameters with an
explicit callable type from the configured imported type export. Use
`effects.<kind>.returnedMembers` to select an exact member call on the receiver
returned by a configured imported factory path, such as `writePool.connect`
with `member: query`. Both are opt-in exact binding selectors; type names,
factory paths, and member names are never inferred. Their categories filter
normally. A returned-member callee displays the configured factory path and
returned member, such as `writePool.connect().query`; a typed executor callee
displays its configured type export, such as `TransactionQuery`.
See [effects configuration](../configuration/effects.md).

Synchronous inline arrows, function expressions, and returned helpers retain their enclosing
invocation context when resolving supplied receiver arguments and defaults. A
later receiver write in that same context does not invalidate an earlier direct
helper call. Calls after a write and captured closures with uncertain execution
order remain unresolved; synthetic reachability edges do not add a second
unknown argument set to a proven invocation. Returned helpers require every
known factory invocation to call its result immediately; a stored result or
opaque escape keeps receiver arguments unresolved. Async and generator helpers
remain conservative because their receiver reads may occur after suspension.


A physical call with multiple known callable possibilities still occupies one
`callSites` row. Only when multiple distinct configured `(callee, category)`
matches remain, the row includes `alternatives: EffectCallAlternative[]`, sorted
by callee and then category. Scalar `callee`/`category` select the first sorted
match deterministically. Ordinary singleton rows omit `alternatives` and retain
their existing shape and spelling-versus-target configuration precedence.
Unknown candidates add no invented effects. Trusted typed/returned executor
provenance is retained alongside matched callable alternatives.

For example, one occurrence alternating between `primary.read` (read) and
`replica.write` (write) produces one row with both alternatives and
`byCategory: { read: 1, write: 1 }`. Each category counts that physical occurrence
once, even if several alternatives match the category; category totals may sum
to more than `callSites.length`. Human and Markdown output append the sorted
alternatives to the same physical row.

`--category read` (or `categories: ["read"]` in the async Node API) projects
candidate evidence before choosing scalar fields and counts. That example then
has scalar `primary.read`/read, one read count, no write evidence, and no
`alternatives` because only one match remains. Category projection preserves
physical source position and occurrence count for selected sites. Existing
single-identity configuration ambiguity is unchanged; this does not infer new
execution ordering or resolve unknown callees.
