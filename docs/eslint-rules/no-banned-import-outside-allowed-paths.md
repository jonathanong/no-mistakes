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

Same-file imported capability calls also include comma callees, optional calls,
and static `.call` / literal-array `.apply` forwarding, including string and
expression-free template member keys. An explicitly banned export named `call`
or `apply` takes precedence over forwarding. Dynamic member keys, spread
`.call` arguments, and opaque, sparse, or spread `.apply` argument arrays are
not resolved through forwarding. Calls still use the existing scope and
assignment tracking, so a shadowed binding does not inherit the import identity.

Another file's re-export is not visible here. `import { load } from "./barrel.mjs"; load(id)` stays unreported when `barrel.mjs` is `export { fetchUser as load } from "@app/data"` or `export * from "@app/data"`. A parameter default such as `function run({ fetchUser } = api)` is not judged, because the caller can pass a different object.

## Suppression

```ts
import { fetchUser } from "@app/data";
// no-mistakes-disable-next-line no-banned-import-outside-allowed-paths -- migration adapter is the reviewed boundary
fetchUser("123");
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` also suppress invocation findings.
Native ESLint directives remain available for import and re-export findings.

## Related rules

- [`no-global-fetch-outside-helper`](no-global-fetch-outside-helper.md) applies
  the same boundary pattern to global `fetch`.
