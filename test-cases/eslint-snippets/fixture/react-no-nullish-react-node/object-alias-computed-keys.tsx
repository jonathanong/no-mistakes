import type { ReactNode } from "react";

type Props = { children: ReactNode; label: string };

export function literal(props: Props) {
  const { ["children"]: slot } = props;
  return slot ?? null;
}

export function asserted(props: Props) {
  const { ["children" as const]: slot } = props;
  return slot ?? null;
}

const childrenKey = "children";

export function constKey(props: Props) {
  const { [childrenKey]: slot } = props;
  return slot ?? null;
}

export function identifier(props: Props, slotName: string) {
  const { [slotName]: slot } = props;
  return slot ?? null;
}

export function template(props: Props) {
  const { [`children${""}`]: slot } = props;
  return slot ?? null;
}

export function otherLiteral(props: Props) {
  const { ["label"]: slot } = props;
  return slot ?? null;
}

export function plain(props: { label: string }, label: string) {
  const { [label]: slot } = props;
  return slot ?? null;
}
