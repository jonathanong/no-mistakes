import type { ReactNode } from "react";
type ReactProps = { children: ReactNode };
type TextProps = { children: string | undefined };

export function forOf(props: ReactProps, values: TextProps[]) {
  const p = props;
  // A loop binding is not the outer React prop alias.
  for (const p of values) {
    const direct = p.children ?? null;
    const { children } = p;
    const destructured = children ?? null;
  }
  return p.children ?? null;
}

export function classic(props: ReactProps, value: TextProps) {
  const p = props;
  for (const p: TextProps = value; false;) {
    const direct = p.children ?? null;
    const { children } = p;
    const destructured = children ?? null;
  }
  return p.children ?? null;
}

export function reverse(props: TextProps, value: ReactProps) {
  const p = props;
  for (const p: ReactProps = value; false;) {
    const direct = p.children ?? null;
    const { children } = p;
    const destructured = children ?? null;
  }
  return p.children ?? null; // Preserve the legacy declared-loop conservative finding.
}

export function caught(props: ReactProps) {
  const p = props;
  try { throw { children: "text" }; } catch (p) {
    const direct = p.children ?? null;
    const { children } = p;
    const destructured = children ?? null;
  }
  return p.children ?? null;
}

export function retainedDeclared(props: { children: string | undefined; footer: ReactNode }, value: ReactProps) {
  // The new footer alias fact must not remove the existing declared children finding.
  for (const p: ReactProps = value; false;) {}
  const p = props;
  return [p.children ?? null, p.footer ?? null];
}
