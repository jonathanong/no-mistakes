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

export function assertedAlias(props: { children: ReactNode }) {
  const textProps = props as { children: string };
  return textProps.children ?? "";
}

export function assertedMember(props: { children: ReactNode }) {
  return (props as { children: string }).children ?? "";
}

export function assertedDestructure(props: { children: ReactNode }) {
  const { children } = props as { children: string };
  return children ?? "";
}

export function assertedNode(props: { children: ReactNode }) {
  const nodeProps = props as { children: ReactNode };
  return nodeProps.children ?? null;
}

export function assertedMissing(props: { children: ReactNode }) {
  const textProps = props as Missing;
  return textProps.children ?? "";
}

declare const moduleName: string;
const Hidden = require(moduleName);

export function annotatedUnknown(props: { children: ReactNode }) {
  const { children }: Record<"children", unknown> = props;
  return children ?? null;
}

type HiddenNode = Hidden.ReactNode;

export function annotatedUnresolved(props: { children: ReactNode }) {
  const { children }: HiddenNode = props;
  return children ?? null;
}

export function annotatedParam({ children }: HiddenNode) {
  return children ?? null;
}

export function namedExpression(node: Hidden.ReactNode) {
  return function node() {
    return node ?? null;
  };
}

export function parameterProperty(node: Hidden.ReactNode) {
  return new (class {
    constructor(private node: string) {
      return node ?? null;
    }
  })();
}
