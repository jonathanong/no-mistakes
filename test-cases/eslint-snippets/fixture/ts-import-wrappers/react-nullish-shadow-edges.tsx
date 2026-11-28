import type { ReactNode } from "react";

interface GetterBase {
  child: ReactNode;
}
interface GetterProps extends GetterBase {
  get child(): string;
}
export function getterOverride(props: GetterProps) {
  return props.child ?? "";
}

interface GetterNode {
  get child(): ReactNode;
}
export function getterNode(props: GetterNode) {
  return props.child ?? null;
}

interface ClassProps {
  child: ReactNode;
}
export function classShadow() {
  class ClassProps {
    child = "";
  }
  const box = new ClassProps();
  return (props: ClassProps) => (box.child ? (props.child ?? "") : "");
}

interface EnumProps {
  child: ReactNode;
}
export function enumShadow() {
  enum EnumProps {
    Child,
  }
  const label = EnumProps.Child;
  return (props: EnumProps) => (label ? (props.child ?? "") : "");
}

type CycleProps = CycleProps;
export function forwardCycle(source: CycleProps) {
  const read = () => child ?? null;
  const { child }: CycleProps = source;
  return read();
}

export function intersectionChild(value: { label: string }) {
  return (value as { child: ReactNode } & { label: string }).child ?? null;
}
export function intersectionLabel(value: { label: string }) {
  return (value as { child: ReactNode } & { label: string }).label ?? null;
}
export function intersectionClash(value: { label: string }) {
  return (value as { child: ReactNode } & { child: string }).child ?? null;
}
