# `no-mistakes/test-no-error-message-matching`

## Why

Error text changes frequently and is often user-facing copy. Tests should prove
the stable error type, code, or structured behavior instead.

## Disallowed

```ts
expect(error.message).toContain("user not found");
```

## Allowed

```ts
expect(error).toBeInstanceOf(NotFoundError);
expect(error.code).toBe("USER_NOT_FOUND");
```

## Options

This rule has no options.

## Fix

Assert an error class, code, status, or other contract field rather than the
message string.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/test-no-error-message-matching -- localization regression explicitly owns this copy
expect(error.message).toBe("Translated error");
```

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) keeps tests isolated.
