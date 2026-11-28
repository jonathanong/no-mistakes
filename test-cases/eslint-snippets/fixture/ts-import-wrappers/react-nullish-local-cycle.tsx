import { ReactNode } from "react";
import R = Other;

interface Outer {
  child: ReactNode;
}

export function localMerge() {
  interface Props {
    label: string;
  }
  interface Props {
    child: ReactNode;
  }
  return (props: Props) => props.child ?? null;
}

export function innerShadowsOuter() {
  interface Outer {
    label: string;
  }
  return (props: Outer) => props.child ?? null;
}

interface Unknown {
  child: R.ReactNode;
}
// The alias reuses the interface name so the child facts can disagree.
interface CycleA {
  child: ReactNode;
}
type CycleA = CycleB;
interface CycleB extends CycleC, Unknown {}
type CycleC = CycleA;

export function cycle(props: CycleA) {
  return props.child ?? null;
}
