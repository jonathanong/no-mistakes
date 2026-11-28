import { ReactNode } from "react";

declare const ready: boolean;
declare const moduleName: string;
const R = require(moduleName);

interface Merged {
  child: ReactNode;
}
interface Merged {
  label: string;
}
export function merged(props: Merged) {
  return props.child ?? null;
}

interface Clash {
  child: ReactNode;
}
interface Clash {
  child: string;
}
export function clash(props: Clash) {
  return props.child ?? null;
}

interface Base {
  child: ReactNode;
}
interface Parts extends Base {}
interface Parts {
  label: string;
}
export function parts(props: Parts) {
  const child = props.child ?? null;
  const label = props.label ?? null;
  return child ?? label;
}

type HiddenProps = { child: R.ReactNode };
type Forwarded = HiddenProps;
export function direct(props: HiddenProps) {
  return props.child ?? null;
}
export function forwarded(props: Forwarded) {
  return props.child ?? null;
}

export function nestedExtends() {
  interface NestedBase {
    child: ReactNode;
  }
  interface NestedProps extends NestedBase {}
  function view(props: NestedProps) {
    return props.child ?? null;
  }
  return view;
}

export function nestedOverride() {
  interface NestedBase {
    child: ReactNode;
  }
  interface NestedProps extends NestedBase {
    child: string;
  }
  function view(props: NestedProps) {
    return props.child ?? null;
  }
  return view;
}

export function loop(props: { a: ReactNode }, source: { a: string; b: ReactNode }) {
  for (const props = source; ready; ) {
    const fromB = props.b ?? null;
    const fromA = props.a ?? null;
    return fromB ?? fromA;
  }
}
