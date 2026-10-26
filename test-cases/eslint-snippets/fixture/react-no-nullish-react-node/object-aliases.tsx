import type { ReactNode } from "react";

type Props = { children: ReactNode; label: string };

export function template(props: Props) {
  const { [`children`]: slot } = props;
  return slot ?? null;
}

export function alias(props: Props) {
  const p = props;
  return p.children ?? null;
}

export function chained(props: Props) {
  const first = props;
  const second = first;
  return second[`children`] ?? null;
}

export function destructuredAlias(props: Props) {
  const p = props;
  const { ["children"]: slot } = p;
  return slot ?? null;
}

export function annotated(children: ReactNode) {
  const props: Props = { children, label: "label" };
  const p = props;
  return p.children ?? null;
}
