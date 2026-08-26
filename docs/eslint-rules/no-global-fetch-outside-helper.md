# `no-mistakes/no-global-fetch-outside-helper`

## Why

Centralizing global `fetch` in a configured helper keeps transport policy,
authentication, retries, and static route analysis in one reviewable place.

## Disallowed

```ts
// web/app/users/page.tsx
await fetch("/api/users");
```

## Allowed

```ts
// web/lib/api/users.ts (an allowed helper path)
export const listUsers = () => fetch("/api/users");
```

## Options

- `checkedPathPatterns` scopes files in which global `fetch` is banned.
- `allowedPathPatterns` exempts the helper paths where global `fetch` may be
  called.

Both lists default to empty, so repositories opt into the boundary explicitly.

## Fix

Move the request into an allowed client/helper module and call that wrapper from
application code.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-global-fetch-outside-helper -- service worker must call fetch directly
await fetch(request);
```

## Related rules

- [`no-banned-import-outside-allowed-paths`](no-banned-import-outside-allowed-paths.md)
  applies the same policy to configured imports.
