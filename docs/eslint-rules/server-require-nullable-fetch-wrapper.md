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

## Fix

Wrap the configured getter within the same function boundary using the required
wrapper, or narrow the configured getter/path policy.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/server-require-nullable-fetch-wrapper -- handler deliberately maps absence to 404 itself
return getUserById(id);
```

## Related rules

- [`no-global-fetch-outside-helper`](no-global-fetch-outside-helper.md)
  centralizes another server I/O boundary.
