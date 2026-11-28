import type { ReactNode } from "react";

// Oxlint rejects a const that redeclares a parameter, and its no-unassigned-vars
// rule flags a later bare `var` of the same name. ESLint keeps both.
export function forwardConstShadow(child: ReactNode) {
  const read = () => child ?? null;
  const child = "text";
  return read();
}

export function shadowedParam(child: ReactNode) {
  const child = "text";
  return child ?? null;
}

export function bareVar(node: ReactNode) {
  var slot = node;
  var slot;
  return slot ?? null;
}
