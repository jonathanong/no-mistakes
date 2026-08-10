# `no-mistakes server edges`

Print server route dependency edges.

```sh
no-mistakes server edges src/server.mts --format json
```

With no roots, prints all configured route-definition and static client-call
edges. With roots and no explicit depth, prints direct edges only. Route nodes
are expanded from configured server roots and mounts; no framework-directory
fallback is inferred.

Node API: `serverRouteEdges(options)`.
