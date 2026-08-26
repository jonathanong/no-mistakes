# `no-mistakes/no-banned-import-outside-allowed-paths`

## Why

Some APIs must be centralized in helpers so permissions, caching, and error
handling stay consistent. Re-exporting a banned binding keeps it reachable.

## Disallowed

```ts
import { fetchUser } from "@app/data";
export { fetchUser } from "@app/data";
```

## Allowed

```ts
// web/lib/api/user.ts is an allowed helper path
import { fetchUser } from "@app/data";
```

## Options

- `checkedPathPatterns` scopes files to inspect.
- `allowedPathPatterns` exempts helper paths.
- `bannedImports` lists `{ `module`, `names` }` entries. The reserved name
  `"default"` bans direct calls to a module's default export, including
  `.default()`.

All lists default to empty, so configure the project policy explicitly.

## Fix

Move the use into an allowed helper and export a project-specific wrapper;
avoid re-exporting the banned binding.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-banned-import-outside-allowed-paths -- migration adapter is the reviewed boundary
import { fetchUser } from "@app/data";
```

## Related rules

- [`no-global-fetch-outside-helper`](no-global-fetch-outside-helper.md) applies
  the same boundary pattern to global `fetch`.
