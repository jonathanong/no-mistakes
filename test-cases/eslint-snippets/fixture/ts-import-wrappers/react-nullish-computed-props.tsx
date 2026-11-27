import type { ReactNode } from "react";

declare const child: unique symbol;

export function symbolKey(props: { [child]: ReactNode }) {
  return props.child ?? null;
}

export function literalTypeKey(props: { ["field"]: ReactNode }) {
  return props.field ?? null;
}

interface Boxed {
  [child]: ReactNode;
  field: ReactNode;
}

export function interfaceSymbol(props: Boxed) {
  return props.child ?? null;
}

export function interfaceField(props: Boxed) {
  return props.field ?? null;
}
