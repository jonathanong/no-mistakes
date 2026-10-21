# `no-mistakes/react-no-use-promise-resolve`

## Why

`React.use(Promise.resolve(value))` creates an immediately resolved promise in
rendering code without providing an async boundary or useful suspension.

## Disallowed

```tsx
import React from "react";

const user = React.use(Promise.resolve(initialUser));
```

## Allowed

```tsx
import React from "react";

const user = initialUser;
const userFromServer = React.use(userPromise);
```

## Options

This rule has no options.

## Fix

Use the value directly, await outside render where appropriate, or pass the
real asynchronous promise to `React.use`.

## Limits

A default or namespace import of `react`, `const { use } = React`, a string or expression-free template key, optional chaining, a comma callee, and `Promise.resolve.call` are the same call. A computed identifier key is not static.

`use` brought in through another file is not visible here.

```tsx
import { use } from "./barrel";
use(Promise.resolve(value));
```

`barrel` may be `export { use } from "react"` or `export * from "react"`.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/react-no-use-promise-resolve -- framework compatibility probe
const value = React.use(Promise.resolve(initialValue));
```

## Related rules

- [`react-no-iife-in-jsx`](react-no-iife-in-jsx.md) keeps render expressions
  analyzable.
