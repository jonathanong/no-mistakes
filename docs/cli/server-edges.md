# `no-mistakes server edges`

Print server route dependency edges.

```sh
no-mistakes server edges src/server.mts --format json
```

With no roots, prints all route edges. With roots and no explicit depth, prints
direct edges only.

Node API: `serverRouteEdges(options)`.
