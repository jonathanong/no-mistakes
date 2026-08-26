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

## Fix

Use a string literal or an expression-free template. Split genuinely dynamic
methods into small static branches when route analysis matters.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-static-fetch-method -- proxy forwards a user-selected method
await fetch(url, { method: requestMethod });
```

## Related rules

- [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) requires the same
  static shape for URLs.
