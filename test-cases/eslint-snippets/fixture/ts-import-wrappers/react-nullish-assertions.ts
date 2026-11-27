import type { ReactNode } from "react";

export function narrowed(child: ReactNode) {
  const text = child as string;
  return text ?? "";
}

export function asserted(value: string) {
  const node = value as ReactNode;
  return node ?? null;
}

export function angle(value: string) {
  const node = <ReactNode>value;
  return node ?? null;
}

export function direct(child: ReactNode) {
  return (child as string) ?? "";
}

export function kept(child: ReactNode) {
  return (child satisfies ReactNode) ?? null;
}

export function nonNull(child: ReactNode) {
  return child! ?? null;
}

export function constAssert(child: ReactNode) {
  const text = child as const;
  return (text as const) ?? null;
}
