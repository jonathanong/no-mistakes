import type { ReactNode } from "react";

// Oxlint rejects this redeclaration; ESLint merges the const onto the parameter.
export function forwardConstShadow(child: ReactNode) {
  const read = () => child ?? null;
  const child = "text";
  return read();
}

export function shadowedParam(child: ReactNode) {
  const child = "text";
  return child ?? null;
}
