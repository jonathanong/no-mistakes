# `no-mistakes/react-no-use-promise-resolve`

## Why

`React.use(Promise.resolve(value))` creates an immediately resolved promise in
rendering code without providing an async boundary or useful suspension.

## Disallowed

```tsx
const user = React.use(Promise.resolve(initialUser));
```

## Allowed

```tsx
const user = initialUser;
const userFromServer = React.use(userPromise);
```

## Options

This rule has no options.

## Fix

Use the value directly, await outside render where appropriate, or pass the
real asynchronous promise to `React.use`.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/react-no-use-promise-resolve -- framework compatibility probe
const value = React.use(Promise.resolve(initialValue));
```

## Related rules

- [`react-no-iife-in-jsx`](react-no-iife-in-jsx.md) keeps render expressions
  analyzable.
