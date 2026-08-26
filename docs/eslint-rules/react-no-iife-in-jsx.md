# `no-mistakes/react-no-iife-in-jsx`

## Why

Immediately invoked functions in JSX hide control flow, complicate rendering
analysis, and make components harder to review.

## Disallowed

```tsx
return <div>{(() => (enabled ? <Save /> : <Cancel />))()}</div>;
```

## Allowed

```tsx
const action = enabled ? <Save /> : <Cancel />;
return <div>{action}</div>;
```

## Options

This rule has no options.

## Fix

Extract the expression into a named variable, helper, or component before the
JSX return.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/react-no-iife-in-jsx -- compact generated fixture
return <div>{(() => label)()}</div>;
```

## Related rules

- [`react-no-nullish-react-node`](react-no-nullish-react-node.md) keeps JSX
  fallback behavior explicit.
