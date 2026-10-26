import type { ReactNode } from "react";
export function render(props: { children: ReactNode }) {
  const { [`children`]: slot } = props;
  const p = props;
  // no-mistakes-disable-next-line react-no-nullish-react-node -- deliberate fallback
  const first = slot ?? null;
  const second = p.children ?? null; // no-mistakes-disable-line react-no-nullish-react-node
  // eslint-disable-next-line no-mistakes/react-no-nullish-react-node
  const third = p.children ?? null;
  return [first, second, third];
}
