# `no-mistakes/no-banned-import-outside-allowed-paths`

## Why

Some APIs must be centralized in helpers so permissions, caching, and error
handling stay consistent. Re-exporting a banned binding keeps it reachable.

## Disallowed

```ts
// web/app/page.ts
import { fetchUser } from "@app/data";
fetchUser("123");
export { fetchUser } from "@app/data";
```

## Allowed

```ts
// web/lib/api/user.ts
import { fetchUser } from "@app/data";
export { fetchUser };
```

## Options

- `checkedPathPatterns` scopes files to inspect.
- `allowedPathPatterns` exempts helper paths.
- `bannedImports` lists `{ `module`, `names` }` entries. The reserved name
  `"default"` bans direct calls to a module's default export, including
  `.default()`.

All lists default to empty, so configure the project policy explicitly.

```js
{
  checkedPathPatterns: ["web/**"],
  allowedPathPatterns: ["web/lib/api/**"],
  bannedImports: [{ module: "@app/data", names: ["fetchUser"] }],
}
```

A string or expression-free template member on a banned binding is the same member.

## Fix

Move the use into an allowed helper and export a project-specific wrapper;
avoid re-exporting the banned binding.

## Limits

Another file's re-export is not visible here. `import { load } from "./barrel.mjs"; load(id)` stays unreported when `barrel.mjs` is `export { fetchUser as load } from "@app/data"` or `export * from "@app/data"`. A parameter default such as `function run({ fetchUser } = api)` is not judged, because the caller can pass a different object. `.call`, `.apply`, and a comma callee are not the banned binding.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-banned-import-outside-allowed-paths -- migration adapter is the reviewed boundary
import { fetchUser } from "@app/data";
```

## Related rules

- [`no-global-fetch-outside-helper`](no-global-fetch-outside-helper.md) applies
  the same boundary pattern to global `fetch`.
