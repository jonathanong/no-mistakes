# `no-mistakes/nextjs-static-fetch-url`

## Why

Static fetch URLs let route analysis connect a Next.js page to the API endpoint
it reaches.

## Disallowed

```ts
await fetch(`/api/users/${userId}`);
```

## Allowed

```ts
await fetch("/api/users");
await fetch(`/api/users`);
```

## Options

This rule has no options.

The rule recognizes global `fetch`, including static member access on
`globalThis`, `window`, `self`, and `global`, and `const` aliases or destructured bindings
of those references. Optional calls are supported.

For `.call` and `.apply`, the rule checks the forwarded URL position. An opaque argument list or a spread that can occupy the URL position
produces a finding for a recognized `fetch` target.

## Fix

Use a literal URL or an expression-free template. Put dynamic routing behind a
small static wrapper when the relationship should be analyzable.

## Limits

`function run({ fetch } = globalThis)` is not judged, because the caller can pass a different function. A computed identifier key is not the `fetch` property, including when that identifier is spelled `fetch`: `const fetch = "setTimeout"; const { [fetch]: schedule } = globalThis`. `const request = fetch as typeof fetch` is still `fetch`. `(globalThis as typeof globalThis).fetch` and `const { fetch: request } = globalThis as typeof globalThis` are still the global `fetch`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-static-fetch-url -- proxy forwards a dynamic resource path
await fetch(resourceUrl);
```

## Related rules

- [`nextjs-static-fetch-method`](nextjs-static-fetch-method.md) requires a
  static HTTP method.
