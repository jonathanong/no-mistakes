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
When both a spelling and a target select one occurrence, the spelling match
keeps its existing category and the occurrence appears once. Reachability
follows import edges from the entry, so a file that `filesystem.skipDirectories`
excludes from discovery can still be reported if it is imported from the entry.
An unknown `<kind>` or a missing entry file is an error.

Node API: `effects(options)`.
