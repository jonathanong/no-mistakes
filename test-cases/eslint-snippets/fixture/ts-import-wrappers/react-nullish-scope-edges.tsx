import type { ReactNode } from "react";

declare const reactModule: string;
const R = require(reactModule);

type Slot = ReactNode;

export function parameterShadow<Slot extends string>(value: Slot) {
  return value ?? "";
}

export function outerSlot(value: Slot) {
  return value ?? "";
}

export function switched(value: string, node: R.ReactNode, flag: boolean) {
  switch (flag) {
    case true:
      const value = node;
      break;
  }
  return value ?? "";
}

export function namedClass(node: R.ReactNode) {
  const C = node;
  return class C {
    static value = C ?? null;
  };
}

export function outerClass(node: R.ReactNode) {
  const C = node;
  return C ?? null;
}

export function siblings() {
  const read = () => value ?? "";
  var value = local;
  var value = local;
  var local = "";
  return read;
}

interface ViewProps {
  child: ReactNode;
}

export function laterDefault(
  props: ViewProps,
  read = () => {
    const alias = props!;
    return alias.child ?? null;
  },
) {
  return read();
}

export function earlierDefault(
  read = () => {
    const alias = props!;
    return alias.child ?? null;
  },
  props: ViewProps,
) {
  return read();
}

interface Base {
  child: ReactNode;
}
interface Quiet extends Base {
  child: ReturnType<() => ReactNode>;
}
interface Plain extends Base {}
interface PropsBoth extends Quiet, Plain {}

export function sharedBase(p: PropsBoth) {
  return p.child ?? null;
}

export function quietOverride(p: Quiet) {
  return p.child ?? null;
}

interface Props {
  child: ReactNode;
}

export function parameterInterface<Props extends { child: number }>(p: Props) {
  return p.child ?? null;
}

export function interfaceProps(p: Props) {
  return p.child ?? null;
}

export function constrainedNode<Props extends { child: ReactNode }>(p: Props) {
  return p.child ?? null;
}

export function forwardLet(node: ReactNode) {
  const read = () => value ?? "";
  let value = node;
  return read();
}

export function reassignedForward(node: ReactNode) {
  const read = () => value ?? "";
  let value = node;
  value = "text";
  return read();
}
