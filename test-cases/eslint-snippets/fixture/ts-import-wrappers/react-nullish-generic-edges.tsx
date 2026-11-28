import type { ReactNode } from "react";

type Slot = ReactNode;

export function caseScalar(flag: boolean) {
  switch (flag) {
    case true:
      type Slot = string;
      const value: Slot = "text";
      return value ?? "";
    default:
      const other = "text";
      return (other as Slot) ?? "";
  }
}

export function caseScalarDiscriminant(flag: boolean) {
  const probe = flag ? "text" : "other";
  switch ((probe as Slot) ?? "") {
    case true:
      type Slot = string;
      return "yes";
  }
  return "no";
}

type DefaultString<T extends ReactNode = string> = T;
export function defaultedGeneric(value: DefaultString) {
  return value ?? "";
}

type DefaultNode<T extends string = ReactNode> = T;
export function defaultedNode(value: DefaultNode) {
  return value ?? "";
}

interface ArgBox<T> {
  child: T;
}
export function suppliedInterfaceNode(value: ArgBox<ReactNode>) {
  return value.child ?? null;
}
export function suppliedInterfaceString(value: ArgBox<string>) {
  return value.child ?? null;
}

interface BoundBox<T extends ReactNode> {
  child: T;
  label: string;
}
export function boundInterfaceString(value: BoundBox<string>) {
  return value.child ?? null;
}
export function boundInterfaceNode(value: BoundBox<ReactNode>) {
  return value.child ?? null;
}
export function boundInterfaceLabel(value: BoundBox<ReactNode>) {
  return value.label ?? null;
}

type LiteralBox<T> = { child: T };
export function literalBoxNode(value: LiteralBox<ReactNode>) {
  return value.child ?? null;
}
export function literalBoxString(value: LiteralBox<string>) {
  return value.child ?? null;
}

namespace React {
  export type ReactNode = string;
}
export function localReactNamespace(value: React.ReactNode) {
  return value ?? null;
}

export function catchDefault(x: ReactNode) {
  try {
    return x ?? null;
  } catch ({ x, y = x ?? "" }) {
    return y ?? null;
  }
}

export function forwardNumber() {
  const read = () => count ?? 0;
  let count = 1;
  count = 2;
  return read();
}

export function forwardNodeWrite(node: ReactNode) {
  const read = () => child ?? null;
  let child = node;
  child = "text";
  return read();
}
