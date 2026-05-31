# `no-mistakes server related`

Print files related through server route edges.

```sh
no-mistakes server related src/routes/users.mts --direction both --format paths
```

Use this to connect mounted routers, route modules, and server entrypoints.

Key option: `--direction deps|dependents|both`.

Node API: `serverRouteRelated(options)`.
