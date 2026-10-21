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

`globalThis`, `window`, `self`, and `global` `fetch`, a string or expression-free template member, a single `const` alias, `const { fetch }` from those roots, an optional call, and `.call` or a literal-array `.apply` are the same call.

## Fix

Move the request into an allowed client/helper module and call that wrapper from
application code.

## Limits

`function run({ fetch } = globalThis)` is not judged, because the caller can pass a different function. `.apply` with a non-literal argument list is not rewritten. A computed identifier key is not the `fetch` property:

```js
const key = "setTimeout";
const { [key]: schedule } = globalThis;
schedule(() => {}, 0);
```

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-global-fetch-outside-helper -- service worker must call fetch directly
await fetch(request);
```

## Related rules

- [`no-banned-import-outside-allowed-paths`](no-banned-import-outside-allowed-paths.md)
  applies the same policy to configured imports.
