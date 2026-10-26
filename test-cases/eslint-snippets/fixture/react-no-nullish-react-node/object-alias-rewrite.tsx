import type { ReactNode } from "react";
export function render(props: { children: ReactNode }) {
  const { [`children`]: slot } = props;
  const p = props;
  return [
    slot === undefined ? null : slot,
    p.children === undefined ? null : p.children,
  ];
}
