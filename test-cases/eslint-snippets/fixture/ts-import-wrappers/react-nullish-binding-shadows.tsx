import type { ReactNode } from "react";
import R from "react";

declare const ready: boolean;

namespace Local {
  export namespace R {
    export type ReactNode = string;
  }
  export function namespaced(child: R.ReactNode) {
    return child ?? "";
  }
}

export function imported(child: R.ReactNode) {
  return child ?? null;
}

export function emptyKey(value: { "": ReactNode }) {
  return value[""] ?? null;
}

export function assertedEmpty(value: { "": string }) {
  return (value as { "": ReactNode })[""] ?? null;
}

export function dynamicKey(value: { child: ReactNode }, key: string) {
  return (value as { child: ReactNode })[key] ?? null;
}

export function closure(child: ReactNode) {
  {
    const read = () => child ?? "";
    const child = "";
    return read;
  }
}

export function laterNode(node: ReactNode) {
  {
    const read = () => child ?? "";
    const child: ReactNode = node;
    return read;
  }
}

export function outerParam(props: { child: string }, source: { child: ReactNode }) {
  for (const props: { child: ReactNode } = source; ready;) {}
  return props.child ?? "";
}

export function insideLoop(source: { child: ReactNode }) {
  for (const props: { child: ReactNode } = source; ready;) {
    return props.child ?? "";
  }
}

export function varLoop(source: { child: ReactNode }) {
  for (var props: { child: ReactNode } = source; ready;) {}
  return props.child ?? "";
}
