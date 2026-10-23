# `no-mistakes/react-no-use-promise-resolve`

## Why

`React.use(Promise.resolve(value))` obscures whether the operand is synchronous
data or an existing asynchronous promise. A synchronous value does not need a
promise created during rendering; an existing promise can be passed directly.

## Disallowed

```tsx
import React from "react";

const user = React.use(Promise.resolve(initialUser));
```

An immutable alias of the global `Promise` constructor has the same issue:

```tsx
import React from "react";
const P = Promise;
const user = React.use(P.resolve(initialUser));
```

## Allowed

```tsx
import React from "react";

const user = initialUser;
const userFromServer = React.use(userPromise);
```

For an alias finding, use a plain value directly when no suspension is needed,
or pass an existing asynchronous promise directly to `React.use`:

```tsx
const value = initialUser;
const userFromServer = React.use(userPromise);
```

## Options

This rule has no options.

## Fix

Use the value directly, await outside render where appropriate, or pass the
real asynchronous promise to `React.use`. Choose the rewrite that matches the
actual operand; the rule does not autofix by removing `Promise.resolve`.

## Limits

A default or namespace import of `react`, `const { use } = React`, a string or expression-free template key, optional chaining, a comma callee, and `Promise.resolve.call` are the same call. `React.use["call"]` forwards the same way as `.call`. A computed identifier key is not static.

The new constructor-alias path follows same-file `const` identity chains to an
unshadowed global `Promise` binding. It requires each alias initializer to precede
its use in source and rejects writes, escaped aliases, namespace member writes,
unknown constructor members, and reads intercepted by a script-mode `with`
object. It also abstains when a global object escapes as a value, a method on
it is called, or `Promise` is accessed through another global-object path in
the same file. A static readonly global property read does not block analysis.
This path handles only an ordinary `React.use(P.resolve(value))` call with one
non-spread operand;
detached `resolve`, `.call`/`.apply`, optional calls, and extra arguments do
not gain new findings. The original direct-`Promise` syntax remains on its
released path.

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
