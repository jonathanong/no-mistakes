# `no-mistakes server related`

Print files related through server route edges.

```sh
no-mistakes server related src/routes/users.mts --direction both --format paths
```

Use this to connect mounted routers, route modules, server entrypoints, direct
client calls, and calls through local or imported route helpers via the
normalized route node.

Key option: `--direction deps|dependents|both`.

Node API: `serverRouteRelated(options)`.
