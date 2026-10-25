import type { ReactNode } from "react";
type ReactProps = { children: ReactNode };
type TextProps = { children: string | undefined };

export function textSource(props: TextProps, value: ReactProps) {
  // The loop binding must not replace the outer parameter's prop facts.
  for (const props: ReactProps = value; false;) {}
  const p = props;
  return p.children ?? null;
}

export function reactSource(props: ReactProps, value: TextProps) {
  for (const props: TextProps = value; false;) {}
  const p = props;
  return p.children ?? null;
}

export function textForOf(props: TextProps, values: ReactProps[]) {
  for (const props of values) {}
  const p = props;
  return p.children ?? null;
}

export function reactForOf(props: ReactProps, values: TextProps[]) {
  for (const props of values) {}
  const p = props;
  return p.children ?? null;
}

export function textForIn(props: TextProps, values: object) {
  for (const props in values) {}
  const p = props;
  return p.children ?? null;
}

export function reactForIn(props: ReactProps, values: object) {
  for (const props in values) {}
  const p = props;
  return p.children ?? null;
}
