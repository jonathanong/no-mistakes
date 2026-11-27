# `no-mistakes/react-no-nullish-react-node`

## Why

React renders `null`, `false`, and empty values differently. `??` on a
ReactNode-like value can accidentally replace an intentional nullish render.

## Disallowed

```tsx
import type { ReactNode } from "react";

export function Panel({ children }: { children: ReactNode }) {
  return <section>{children ?? <EmptyState />}</section>;
}

export function AliasedPanel(props: { children: ReactNode }) {
  const { [`children`]: slot } = props;
  const p = props;
  return [slot ?? null, p.children ?? null];
}
```

## Allowed

```tsx
import type { ReactNode } from "react";

export function Panel({ children }: { children: ReactNode }) {
  return <section>{children === undefined ? <EmptyState /> : children}</section>;
}

export function AliasedPanel(props: { children: ReactNode }) {
  const { [`children`]: slot } = props;
  const p = props;
  return [slot === undefined ? null : slot, p.children === undefined ? null : p.children];
}
```

## Options

`unanalyzable` is `"finding"` or `"ignore"`. It defaults to `"finding"`. A
ReactNode type whose import cannot be resolved reports `unanalyzable` instead
of passing silently. `"ignore"` drops only that uncertain report.

## Fix

Check `undefined` explicitly when only an omitted prop should receive a
fallback; preserve an explicit `null` when it is meaningful.

## Limits

`props["children"]`, ``props[`children`]``, and `const node = children` are the same ReactNode value when the binding is declared in this file. A computed identifier key is not a static property name.

`import React = require("react")`, `import { default as React } from "react"`, `import { "default" as React } from "react"`, `let React = require("react")`, `import("react").ReactNode`, `(children satisfies ReactNode)`, `props!.children`, and `const { children } = props!` are the same ReactNode value. A non-static `const React = require(moduleName)`, `let React = require(moduleName)`, or `import(moduleName).ReactNode`, or a later write to that `let`, reports `unanalyzable`. A named interface property of that type does too, including through a `const` alias of the object and a further `const` alias of that alias. A shadowing parameter stops the lookup. A local function named `require` is not the module loader. A variable with its own type annotation is not inferred from the initializer, so `const text: string = children as string` is not a ReactNode. An `as` or angle-bracket assertion uses that type instead of the source, so `const text = child as string` and `(child as string) ?? ""` are not ReactNode, and `const node = value as ReactNode` is. `as const`, `satisfies`, and `!` still read the inner value.

String and expression-free template destructuring keys retain the declared prop
type. An unannotated `const p = props` also retains it, including through more
const aliases, when the source is an unchanged parameter or an already initialized
const. Reassigned bindings, `let` and `var` aliases, parameter-default aliases,
and runtime expressions do not establish that prop type. An alias with its own
explicit type uses that contract instead. An alias in a parameter initializer is
also left unknown.
An alias source and reads of that alias use its lexical binding's prop type; a
same-named loop or catch local does not inherit the alias's type. Declared-type
name facts keep their existing conservative reporting. A computed destructuring
key counts only when it is a static literal, so an identifier key does not take
the ReactNode prop spelled like that identifier. A computed key in a type
literal or interface follows the same rule. A rest element, a name nested
in an object or array pattern, an array-pattern parameter, a rest parameter,
and `const [name] = ...` stop the lookup and do not inherit an outer ReactNode.

```tsx
import type { ReactNode } from "react";
export function Unknown(props: { children: ReactNode }, children: string) {
  const { [children]: slot } = props;
  return slot ?? null; // The runtime key need not be "children".
}
```

A prop type imported through another file's re-export is not visible here.

```tsx
import type { PanelProps } from "./barrel";
export const Panel = (props: PanelProps) => props.footer ?? <Empty />;
```

`barrel` may be `export type { PanelProps } from "./types"` where `footer` is `ReactNode`.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/react-no-nullish-react-node -- API contract equates null and omission here
return <section>{children ?? <EmptyState />}</section>;
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names
`react-no-nullish-react-node` or `no-mistakes/react-no-nullish-react-node`.
Directive-looking string literals do not.

## Related rules

- [`ts-preserve-null-option-defaults`](ts-preserve-null-option-defaults.md)
  applies the same null-preservation principle to options.
