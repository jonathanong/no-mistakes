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
```

## Allowed

```tsx
import type { ReactNode } from "react";

export function Panel({ children }: { children: ReactNode }) {
  return <section>{children === undefined ? <EmptyState /> : children}</section>;
}
```

## Options

This rule has no options.

## Fix

Check `undefined` explicitly when only an omitted prop should receive a
fallback; preserve an explicit `null` when it is meaningful.

## Limits

`props["children"]`, `props[\`children\`]`, and `const node = children` are the same ReactNode value when the binding is declared in this file. A computed identifier key is not a static property name.

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
