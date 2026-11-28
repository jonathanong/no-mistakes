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

`unanalyzable` is `"finding"` or `"ignore"`. It defaults to `"finding"`. A call
that looks like `use(Promise.resolve(...))` whose `use` import cannot be
resolved reports `unanalyzable` instead of passing silently. `"ignore"` drops
only that uncertain report.

## Fix

Use the value directly, await outside render where appropriate, or pass the
real asynchronous promise to `React.use`. Choose the rewrite that matches the
actual operand; the rule does not autofix by removing `Promise.resolve`.

## Limits

A default or namespace import of `react`, `const { use } = React`, `require("react")`, `import React = require("react")`, a string or expression-free template key, optional chaining, a comma callee, and `Promise.resolve.call` are the same call. `React.use["call"]` forwards the same way as `.call`, and so does a `const` bound to `"call"` or `"apply"`. A computed key on a React namespace or an unresolved module reports `unanalyzable`, including `const { [key]: use } = React` when `key` is not a static string. A `const` identifier bound to a string literal is that key, including through `as` and a computed destructuring key. A name bound by array or object destructuring is not that whole initializer. A later assignment of `use` to that binding is that call when the pattern itself is not an import. A repeated `var` initialized from `require("react").use` reports `unanalyzable`, and one declaration of that member stays `use`. A computed key on any other value is not static. `as`, `satisfies`, and a type-argument instantiation on `use` or `Promise.resolve` do not hide the call. A type-only import is not a runtime `use`. A type-only `import type { require }` does not shadow the loader. An inner type named `require` does not hide an outer runtime `require`. A non-static `require` of the callee reports `unanalyzable`. `import { default as React }` is the default import. `const P = Promise as typeof Promise` still reaches `P.resolve`, including `typeof globalThis.Promise`. A type query is not a runtime use of the global object. A local overload named `use` is not the React import. A const alias of `React.use` stays that call when an inner binding reuses the alias name. A later write to a destructured binding is `unanalyzable`. A `let` or `var` alias of `use` with no later write is that call, and a later write reports `unanalyzable`. A dynamic member of a type-only namespace is not a runtime `use`. `let React = require("react")` is that namespace, and a later write to that `let` reports `unanalyzable`. A `let` or `var` reassigned between local functions, including the same function twice, is not `use`. `var left = right?.x; var right = left?.y` reports `unanalyzable`. A later assignment of an import, or a write whose value cannot be read, reports `unanalyzable`. A destructuring default reports `unanalyzable` when it is not the same import as the property. An inner type does not hide an outer value import of the same name. Repeated `var` bindings assigned an import report `unanalyzable`. An assignment after a call does not make that call the import.

The constructor-alias path follows same-file `const` identity chains to global
`Promise` when no local binding shadows it. It requires each alias initializer to precede
its use in source and rejects writes, escaped aliases, namespace member writes,
unknown constructor members, and reads intercepted by a script-mode `with`
object. It also abstains when a global object escapes as a value, a method on
it is called, or `Promise` is accessed through another global-object path in
the same file. A static readonly global property read does not block analysis.
This path handles an ordinary `React.use(P.resolve(value))` call and the same
alias passed through `use.call` or a one-element `use.apply`. Optional calls,
spreads, extra arguments, and a non-literal `.apply` list do not gain new
findings. A spread before the forwarded argument of `call` or `apply` does not
select a later argument. `require("react").Children.use` is not `use`. The original
direct-`Promise` syntax remains on its released path.

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

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names
`react-no-use-promise-resolve` or `no-mistakes/react-no-use-promise-resolve`.
Directive-looking string literals do not.

## Related rules

- [`react-no-iife-in-jsx`](react-no-iife-in-jsx.md) keeps render expressions
  analyzable.
