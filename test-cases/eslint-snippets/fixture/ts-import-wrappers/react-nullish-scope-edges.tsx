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

export function staticMethod(child: ReactNode) {
  return class Box {
    static {
      const child = 1;
    }
    render() {
      return child ?? null;
    }
  };
}

export function staticInner(child: ReactNode) {
  class Box {
    static {
      const child = 1;
      const read = () => child ?? null;
      read();
    }
  }
  return child ?? null;
}

export function forwardDestructure(props: { child: ReactNode }) {
  const read = () => child ?? null;
  const { child } = props;
  return read();
}

export function forwardPlain(props: { child: string }) {
  const read = () => child ?? null;
  const { child } = props;
  return read();
}

import type { ReactNode as SlotFace } from "react";

export function recursiveInterface() {
  interface SlotFace {
    child: SlotFace;
  }
  const read = (p: SlotFace) => p.child ?? null;
  return read;
}

interface EmptyNamed {
  "": ReactNode;
}
export function namedEmpty(p: EmptyNamed) {
  return p[""] ?? null;
}

export function destructureEmpty({ [""]: child }: { "": ReactNode }) {
  return child ?? null;
}

interface PendingOrder {
  child: R.ReactNode;
}
interface KnownOrder extends BaseOrder {}
interface BaseOrder {
  child: ReactNode;
}
interface PropsOrder extends PendingOrder, KnownOrder {}
export function inheritedOrder(p: PropsOrder) {
  return p.child ?? null;
}

type BoxedProps<Props> = Props;
export function boxedString(p: BoxedProps<{ child: string }>) {
  return p.child ?? "";
}
type KeptProps<T> = Props;
export function keptProps(p: KeptProps<string>) {
  return p.child ?? null;
}
type SameProps = Props;
export function samePropsAlias(p: SameProps) {
  return p.child ?? null;
}

export import ReactReq = require("react");
export function exportedRequire(x: ReactReq.ReactNode) {
  return x ?? null;
}

export function annotatedForward() {
  const read = () => child ?? null;
  let child: ReactNode = null;
  child = "text";
  return read();
}

export function annotatedNumber() {
  const read = () => child ?? null;
  let child: number = 0;
  child = 1;
  return read();
}

export function localSelf() {
  type Props = Props;
  const read = (p: Props) => p.child ?? null;
  return read;
}
