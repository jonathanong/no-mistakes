import type { ReactNode } from "react";

declare const reactModule: string;
const R = require(reactModule);

interface Outer {
  child: ReactNode;
}

export function fromOuter(value: Outer) {
  interface Inner extends Outer {
    label: string;
  }
  const node: Inner = value;
  return node.child ?? null;
}

export function fromUnresolved() {
  interface Base {
    child: R.ReactNode;
  }
  interface Child extends Base {
    label: string;
  }
  const node: Child = { label: "x", child: null };
  return node.child ?? null;
}

export function fromAlias() {
  type Alias = { child: ReactNode };
  interface Uses extends Alias {
    label: string;
  }
  const node: Uses = { label: "x", child: null };
  return node.child ?? null;
}
