# `no-mistakes/server-require-nullable-fetch-wrapper`

## Why

Nullable entity getters need one configured wrapper so a missing entity maps to
the same `null` or response behavior in every server handler.

## Disallowed

```ts
export async function getUserRoute() {
  return getUserById(id);
}
```

## Allowed

```ts
export async function getUserRoute() {
  return nullableEntity(getUserById(id));
}
```

## Options

- `getterCalleePatterns` and `requiredWrapperCallee` are required.
- `includePathPatterns` and `excludePathPatterns` scope matching files.
- `nullableReturnTypeNames` adds nullable return-type hints.
- `inferNullableFromTopLevelEntityPath` and `topLevelEntityPathPatterns`
  enable path-based nullable inference.

Optional lists default to empty and inference defaults to `false`.

An import rename is spelled back to the imported name before the getter pattern is tested. An optional call, a comma callee, and `.call` or a literal-array `.apply` inside the wrapper are the same call.

## Fix

Wrap the configured getter within the same function boundary using the required
wrapper, or narrow the configured getter/path policy.

## Limits

A callee whose own name is `call` or `apply`, such as `rpc.call("/users/1")` or `rpc.call(...args)`, is matched as that callee before forwarding is considered. `requiredWrapperCallee` matches the local binding, so `import { nullableEntity as wrap }` with `requiredWrapperCallee: "wrap"` still accepts `wrap(getUser())`. The imported name `nullableEntity` matches as well.

Another file's re-export is not visible here. `import { get } from "./barrel.mjs"` stays unreported when `barrel.mjs` is `export { serverApi as get } from "./server-api"` or `export * from "./server-api"`. A parameter default such as `function run({ get } = serverApi)` is not judged, because the caller can pass a different object. `.apply` with a non-literal argument list is not rewritten.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/server-require-nullable-fetch-wrapper -- handler deliberately maps absence to 404 itself
return getUserById(id);
```

## Related rules

- [`no-global-fetch-outside-helper`](no-global-fetch-outside-helper.md)
  centralizes another server I/O boundary.
