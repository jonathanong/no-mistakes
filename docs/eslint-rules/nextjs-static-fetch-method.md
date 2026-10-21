# `no-mistakes/nextjs-static-fetch-method`

## Why

Static HTTP methods let route and fetch analysis determine page-to-API
relationships without executing the application.

## Disallowed

```ts
await fetch("/api/users", { method: requestMethod });
```

## Allowed

```ts
await fetch("/api/users", { method: "POST" });
await fetch("/api/users", { method: `POST` });
```

## Options

This rule has no options.

`globalThis`, `window`, `self`, and `global` `fetch`, a string or expression-free template member, a single `const` alias, `const { fetch }` from those roots, an optional call, and `.call` or a literal-array `.apply` are the same call. `.call` and `.apply` shift the arguments.

## Fix

Use a string literal or an expression-free template. Split genuinely dynamic
methods into small static branches when route analysis matters.

## Limits

`function run({ fetch } = globalThis)` is not judged, because the caller can pass a different function. `.apply` with a non-literal argument list is not rewritten. A computed identifier key is not the `fetch` property, including when that identifier is spelled `fetch`: `const fetch = "setTimeout"; const { [fetch]: schedule } = globalThis`. `const request = fetch as typeof fetch` is still `fetch`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-static-fetch-method -- proxy forwards a user-selected method
await fetch(url, { method: requestMethod });
```

## Related rules

- [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) requires the same
  static shape for URLs.
