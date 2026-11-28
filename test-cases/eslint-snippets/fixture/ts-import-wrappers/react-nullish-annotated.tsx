import type { ReactNode } from "react";
export function view(children: ReactNode) {
  const text: string = children as string;
  return text ?? "";
}
