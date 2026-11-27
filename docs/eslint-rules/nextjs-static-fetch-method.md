# `no-mistakes/nextjs-static-fetch-method`

## Why

Static HTTP methods let route and fetch analysis determine page-to-API
relationships without executing the application.

## Disallowed

```ts
await fetch("/api/users", { method: requestMethod });
await fetch("/api/users", { method: "GET", ...overrides });
await fetch.apply(undefined, args);
```

## Allowed

```ts
await fetch("/api/users", { method: "POST" });
await fetch("/api/users", { method: `POST` });
await fetch("/api/users", { ...overrides, method: "GET" });
```

## Options

This rule has no options.

The rule recognizes global `fetch`, including static member access on
`globalThis`, `window`, `self`, and `global`, and `const` aliases or destructured bindings
of those references. Optional calls are supported.

For `.call` and `.apply`, the rule checks the forwarded options position. It reports a recognized `fetch` target when an opaque forwarding list
or an argument spread hides the options position.

Within an options object literal, properties take effect in source order.
A spread or computed key that could set `method` makes it unresolved. A later
explicit literal `method` takes precedence. A call with omitted options or an
object literal with no possible `method` property is allowed.

## Fix

Use a string literal or an expression-free template. Split genuinely dynamic
methods into small static branches when route analysis matters.

## Limits

The object property check applies to inline object literals. Pass an inline
object to make the method visible when options are assembled elsewhere.

`function run({ fetch } = globalThis)` is not judged, because the caller can pass a different function. A computed identifier key is not the `fetch` property, including when that identifier is spelled `fetch`: `const fetch = "setTimeout"; const { [fetch]: schedule } = globalThis`. `const request = fetch as typeof fetch` is still `fetch`. `(globalThis as typeof globalThis).fetch` and `const { fetch: request } = globalThis as typeof globalThis` are still the global `fetch`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-static-fetch-method -- proxy forwards a user-selected method
await fetch(url, { method: requestMethod });
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not.

## Related rules

- [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) requires the same
  static shape for URLs.
