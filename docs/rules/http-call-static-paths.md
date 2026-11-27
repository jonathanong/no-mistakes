# `http-call-static-paths`

Configures the path prefixes that select literal HTTP client calls for `http` edges.

## Why and when

Use this rule when callers should connect to backend routes only through literal paths under known prefixes. It does not emit a `check` finding. A dynamic call produces no `http` edge. Empty prefixes leave prefix matching on `route-consistency`'s `backendPrefixes`.

## What it requires

A call is eligible when it is `fetch(literal)` or `<receiver>.<verb>(literal)` for `get`, `post`, `put`, `patch`, `delete`, `head`, or `options`, and the path starts with a resolved prefix. A template literal counts only when it has no interpolations. Any other path is skipped.

When `backendPrefixes` is non-empty, those prefixes replace `route-consistency`'s `backendPrefixes` for this match. When it is empty, the `route-consistency` list is used. If the resolved list is empty, no HTTP calls are extracted.

An eligible call still needs a matching route definition from [`http-route-static-paths`](http-route-static-paths.md), from `route-consistency` when those HTTP route fields are empty, or from a Next `route.*` file under `route-consistency`'s `frontendRoot`. No definition means no `http` edge.

## Options

`backendPrefixes` defaults to an empty list. Unknown option keys fail configuration validation.

```yaml
rules:
  - rule: http-call-static-paths
    scope: repository
    options:
      backendPrefixes:
        - /http/
```

| Option | Default | Effect |
| --- | --- | --- |
| `backendPrefixes` | `[]` | Path prefixes. Non-empty replaces `route-consistency`'s `backendPrefixes` for HTTP calls. Empty keeps that list. |

## Valid example

```ts
client.get("/http/users/42");
fetch("/http/users");
```

Both paths start with `/http/` and contain no interpolation, so they can match a literal `/http/users/:id` registration.

## Counterexample

```ts
client.get(`/http/users/${id}`);
fetch(url);
```

Neither call has a static path, so neither produces an `http` edge. `check` does not report the call.

A literal path outside every resolved prefix is also skipped.

## Fix

Pass a string literal, or a template with no interpolations, whose path starts with a configured prefix and matches a static backend registration. Set `backendPrefixes` when HTTP calls should use a different prefix list than `route-consistency`.

## Suppression

This rule does not emit a line finding. A `no-mistakes-disable-next-line` or `no-mistakes-disable-file` comment does not add an `http` edge for a dynamic call or remove one recorded for a literal call. Change the call or the prefix list.

## Related rules

[`http-route-static-paths`](http-route-static-paths.md) supplies the backend definitions these calls match. [`route-consistency`](route-consistency.md) is the fallback for prefixes, the backend pattern, and the register object, and it still owns `route` edges. Edge kinds are listed in [graph edges](../graph-edges.md).
