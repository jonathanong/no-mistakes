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
  return [
    slot === undefined ? null : slot,
    p.children === undefined ? null : p.children,
  ];
}
```

## Options

This rule has no options.

## Fix

Check `undefined` explicitly when only an omitted prop should receive a
fallback; preserve an explicit `null` when it is meaningful.

## Limits

`props["children"]`, ``props[`children`]``, and `const node = children` are the same ReactNode value when the binding is declared in this file. A computed identifier key is not a static property name.

String and expression-free template destructuring keys retain the declared prop
type. An unannotated `const p = props` also retains it, including through more
const aliases, when the source is an unchanged parameter or an already initialized
const. Reassigned bindings, `let` and `var` aliases, parameter-default aliases,
and runtime expressions do not establish that prop type. An alias with its own
explicit type uses that contract instead. An alias in a parameter initializer is
also left unknown.
An alias source and reads of that alias use its lexical binding's prop type; a
same-named loop or catch local does not inherit the alias's type. Declared-type
name facts keep their existing conservative reporting. Computed identifier
destructures also retain conservative reporting by the identifier's spelling;
that spelling does not prove the runtime key.

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

## Related rules

- [`ts-preserve-null-option-defaults`](ts-preserve-null-option-defaults.md)
  applies the same null-preservation principle to options.
