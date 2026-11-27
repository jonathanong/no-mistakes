# `http-route-static-paths`

Configures the backend files and register object used to build `http` edges.

## Why and when

Use this rule when HTTP call edges should bind to a different route module than the one `route-consistency` uses for `route` edges. It does not emit a `check` finding. Empty options leave HTTP route extraction on the `route-consistency` backend fields.

## What it requires

When `backendPattern` and `registerObject` are both non-empty, HTTP route extraction uses those values instead of `route-consistency`'s `backendPattern` and `backendRegisterObject`. If either field is empty, HTTP extraction keeps the `route-consistency` value for that field. An invalid glob installs no extractor from this rule.

Definitions are literal verb registrations on `registerObject` in files that match the glob, the same shape `route-consistency` reads. A non-literal path contributes no definition. Next `route.ts` handlers are still taken from `route-consistency`'s `frontendRoot` when that field is set. `route` edges ignore this rule and keep reading `route-consistency`.

## Options

Both fields default to empty strings. Unknown option keys fail configuration validation.

```yaml
rules:
  - rule: http-route-static-paths
    scope: repository
    options:
      backendPattern: http/**/*.mts
      registerObject: httpApp
```

| Option | Default | Effect |
| --- | --- | --- |
| `backendPattern` | `""` | Glob of backend files for HTTP route definitions. Empty keeps `route-consistency`'s `backendPattern`. Invalid installs no extractor from this rule. |
| `registerObject` | `""` | Binding that receives the literal verb calls. Empty keeps `route-consistency`'s `backendRegisterObject`. |

## Valid example

```ts
httpApp.get("/http/users/:id", handler);
```

With the options above, a literal HTTP call whose path matches `/http/users/:id` can connect to this file. The `route-consistency` register object is left unchanged for `route` edges.

## Counterexample

```ts
httpApp.get(path, handler);
```

The call is not a static registration, so this file contributes no HTTP route definition. Calls that would have matched a literal path produce no `http` edge to it.

## Fix

Register the path as a string literal on `registerObject`, and set `backendPattern` to a glob that matches that file. Leave both fields empty to keep using the `route-consistency` backend settings for HTTP definitions.

## Suppression

This rule does not emit a line finding. A `no-mistakes-disable-next-line` or `no-mistakes-disable-file` comment does not invent a missing HTTP route definition or hide one that was extracted. Change the registration or the options.

## Related rules

[`route-consistency`](route-consistency.md) still owns `route` edges and is the fallback for these two fields. [`http-call-static-paths`](http-call-static-paths.md) chooses which literal client calls become `http` edges. Edge kinds are listed in [graph edges](../graph-edges.md).
