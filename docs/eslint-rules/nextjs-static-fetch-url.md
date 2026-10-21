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

`globalThis`, `window`, `self`, and `global` `fetch`, a string or expression-free template member, a single `const` alias, `const { fetch }` from those roots, an optional call, and `.call` or a literal-array `.apply` are the same call. `.call` and `.apply` shift the arguments.

## Fix

Use a literal URL or an expression-free template. Put dynamic routing behind a
small static wrapper when the relationship should be analyzable.

## Limits

`function run({ fetch } = globalThis)` is not judged, because the caller can pass a different function. `.apply` with a non-literal argument list is not rewritten. A computed identifier key (`const { [key]: schedule } = globalThis`) is not the `fetch` property.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-static-fetch-url -- proxy forwards a dynamic resource path
await fetch(resourceUrl);
```

## Related rules

- [`nextjs-static-fetch-method`](nextjs-static-fetch-method.md) requires a
  static HTTP method.
