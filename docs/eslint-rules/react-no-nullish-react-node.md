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

`props["children"]`, ``props[`children`]``, and `const node = children` are the same ReactNode value when the binding is declared in this file. A computed identifier key is not a static property name. An empty string key is that property. A numeric literal key such as `[0]` is the property `"0"`.

`import React = require("react")`, `import { default as React } from "react"`, `import { "default" as React } from "react"`, `let React = require("react")`, `var React = require("react")`, `import("react").ReactNode`, `(children satisfies ReactNode)`, `props!.children`, and `const { children } = props!` are the same ReactNode value. A non-static `const React = require(moduleName)`, `let React = require(moduleName)`, `var React = require(moduleName)`, or `import(moduleName).ReactNode`, a later write to that `let` or `var`, or `import React = Other` used as `React.ReactNode`, reports `unanalyzable`. An inner `type Slot = string` does not keep an outer unresolved `Slot`. `type Value = Plain | Plain` is not a cycle, and `type Left = Right; type Right = Left` reports `unanalyzable`, including when both aliases are exported. A direct `type Slot = Slot` reports `unanalyzable`, including `type Loop<T> = Loop`. `type Boxed<Boxed> = Boxed` stays the type parameter. A named interface property of that type does too, including through a `const` alias of the object and a further `const` alias of that alias. A shadowing parameter stops the lookup. A local function named `require` is not the module loader. A variable with its own type annotation is not inferred from the initializer, so `const text: string = children as string` is not a ReactNode. `const { children }: Record<"children", unknown> = props` is not ReactNode either, and an annotation the rule cannot resolve reports `unanalyzable`, including a destructured parameter. An interface property that overrides an inherited ReactNode uses the override, a named function expression is that function rather than an outer parameter, a block-scoped function declaration is that function rather than an outer parameter, a constructor parameter property is that parameter rather than an outer binding of the same name, a nested type of the same name is that declaration, and a catch parameter is that parameter rather than an outer binding. An `as` or angle-bracket assertion uses that type instead of the source, so `const text = child as string` and `(child as string) ?? ""` are not ReactNode, and `const node = value as ReactNode` is. `as const`, `satisfies`, and `!` still read the inner value. `const textProps = props as { children: string }`, `(props as { children: string }).children`, and `const { children } = props as { children: string }` use that asserted type, and an assertion the rule cannot classify does not fall back to the source props. A nested interface keeps ReactNode props from the interface it extends, and a local property overrides that base. Two declarations of one interface merge, including every declaration in the nearest scope; a property they classify differently is unanalyzable, and linting still finishes. An alias cycle with an inherited property is unanalyzable too. A type alias of an object type keeps that object's props. A loop binding keeps only that binding's props. A loop const does not replace an outer binding of the same name, and its props do not carry past the loop. A class declaration shadows an outer binding. A closure uses a later binding of the same name, including a property taken from a later destructure. A class static block keeps its const bindings inside that block. A `var` in that block stays there and does not replace an outer parameter of the same name. A member whose type is the enclosing interface is that interface, not an outer import of the same name. An empty string key stays on a named interface and on a computed destructure. A later definite inherited ReactNode replaces an earlier unresolved fact. A generic alias whose target is its own type parameter does not copy a same-named interface. `export import ReactReq = require("react")` is that namespace. An annotated binding keeps that annotation when it is reassigned. A local `type Props = Props` does not inherit an outer interface's props. An `export interface` declared in a namespace is that interface. A local namespace qualifier is that namespace, so nested `R.ReactNode` is not the imported React type. A function type parameter shadows an outer alias, and its constraint is the type. A `const` in a `switch` does not replace an outer binding. A named class expression's name is that class inside its body. A `const` alias in a later parameter default can read an earlier parameter, and a default cannot read a parameter that is not initialized yet. A type parameter shadows an interface of the same name. Each extended interface still sees a shared base. Sibling `var` initializers do not leave a finished binding in the seen set. A forward binding with a later write is `unanalyzable` instead of the initializer's type, while a binding with its own type annotation keeps that annotation. A `let` inferred as ReactNode becomes `unanalyzable` when a later write replaces it. A forward initializer uses the binding it names, so a closure parameter that shadows that name does not change it.

String and expression-free template destructuring keys retain the declared prop
type. An unannotated `const p = props` also retains it, including through more
const aliases, when the source is an unchanged parameter or an already initialized
const. Reassigned bindings, `let` and `var` aliases, parameter-default aliases,
and runtime expressions do not establish that prop type. An alias with its own
explicit type uses that contract instead. An alias in a parameter initializer is
also left unknown.
An alias source and reads of that alias use its lexical binding's prop type; a
same-named loop or catch local does not inherit the alias's type. A loop declaration does not publish its props onto a later binding of the same name. A computed destructuring
key counts only when it is a static literal, including a numeric literal, `as const`, and a `const`
string binding. An identifier key does not take the ReactNode prop spelled like
that identifier. When that object has a ReactNode or unresolved prop, a key
that is not static is `unanalyzable`. A computed key in a type
literal or interface counts only when it is a static literal. A numeric literal in that position is the property name. A rest element, a name nested
in an object or array pattern, an array-pattern parameter, a rest parameter,
and `const [name] = ...` stop the lookup and do not inherit an outer ReactNode.

```tsx
import type { ReactNode } from "react";
export function Unknown(props: { children: ReactNode }, children: string) {
  const { [children]: slot } = props;
  return slot ?? null; // unanalyzable: the runtime key need not be "children".
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
