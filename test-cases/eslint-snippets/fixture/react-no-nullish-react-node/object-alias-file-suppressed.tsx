// no-mistakes-disable-file react-no-nullish-react-node -- deliberate fallback
import type { ReactNode } from "react";
export function render(props: { children: ReactNode }) {
  const { [`children`]: slot } = props;
  const p = props;
  return [slot ?? null, p.children ?? null];
}
