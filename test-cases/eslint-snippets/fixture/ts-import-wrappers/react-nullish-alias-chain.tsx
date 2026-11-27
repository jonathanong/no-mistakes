import type { ReactNode } from "react";

declare const reactModule: string;
const R = require(reactModule);

interface Known {
  children: ReactNode;
}

interface Unknown {
  children: R.ReactNode;
}

export function known(props: Known) {
  const first = props;
  const second = first;
  return second.children ?? null;
}

export function unknown(props: Unknown) {
  const first = props;
  const second = first;
  return second.children ?? null;
}
