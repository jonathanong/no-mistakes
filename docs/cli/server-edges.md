# `no-mistakes server edges`

Print server route dependency edges.

```sh
no-mistakes server edges src/server.mts --format json
```

With no roots, prints all configured route-definition and static client-call
edges. With roots and no explicit depth, prints direct edges only. Route nodes
are expanded from configured server roots and mounts; no framework-directory
fallback is inferred. Client-call sources honor `--filter` and configured test
exclusions; static local and imported route-helper calls are resolved from the
same prepared facts.

Filters select client-call source files, not helper modules: an included client
can still resolve a static local or imported helper omitted by the filter.
Configured route roots and mounts continue to define server-route definitions.
Configured Python, Go, Rust, Rails, PHP, Java, Kotlin, Elixir, and .NET client
root-relative literals appear as `client-call` edges to the matching route
pattern. Host-qualified and computed URLs produce no edge. Test files are
excluded the same way as other client sources.

Node API: `serverRouteEdges(options)`.
