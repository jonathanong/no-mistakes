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

## Fix

Use a literal URL or an expression-free template. Put dynamic routing behind a
small static wrapper when the relationship should be analyzable.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-static-fetch-url -- proxy forwards a dynamic resource path
await fetch(resourceUrl);
```

## Related rules

- [`nextjs-static-fetch-method`](nextjs-static-fetch-method.md) requires a
  static HTTP method.
