# `no-mistakes/react-no-nullish-react-node`

## Why

React renders `null`, `false`, and empty values differently. `??` on a
ReactNode-like value can accidentally replace an intentional nullish render.

## Disallowed

```tsx
return <section>{children ?? <EmptyState />}</section>;
```

## Allowed

```tsx
return <section>{children === undefined ? <EmptyState /> : children}</section>;
```

## Options

This rule has no options.

## Fix

Check `undefined` explicitly when only an omitted prop should receive a
fallback; preserve an explicit `null` when it is meaningful.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/react-no-nullish-react-node -- API contract equates null and omission here
return <section>{children ?? <EmptyState />}</section>;
```

## Related rules

- [`ts-preserve-null-option-defaults`](ts-preserve-null-option-defaults.md)
  applies the same null-preservation principle to options.
