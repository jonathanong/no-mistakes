import type { ReactNode } from "react";

export function restShadow(p: { child: ReactNode }) {
  return function inner({ ...p }: { child: string }) {
    return p.child ?? "";
  };
}

export function nestedShadow(child: ReactNode) {
  return function inner({ nested: { child } }: { nested: { child: string } }) {
    return child ?? "";
  };
}

export function nestedArray(child: ReactNode) {
  return function inner({ pair: [child] }: { pair: [string] }) {
    return child ?? "";
  };
}

export function arrayShadow(child: ReactNode) {
  return function inner([child]: [string]) {
    return child ?? "";
  };
}

const field = "text";

export function computedKey({ [field]: value }: { field: ReactNode; text: string }) {
  return value ?? null;
}

export function literalKey({ ["field"]: value }: { field: ReactNode; text: string }) {
  return value ?? null;
}

export function restParam(child: ReactNode) {
  return (...child: string[]) => child ?? [];
}

export function arrayLocal(child: ReactNode) {
  return function inner() {
    const [child] = [""];
    return child ?? "";
  };
}
