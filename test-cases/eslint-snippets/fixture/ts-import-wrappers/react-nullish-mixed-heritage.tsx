import type { ReactNode } from "react";

// The alias reuses the interface name, so this file stays off the oxlint list.
export function fromMixed() {
  type Both = { child: string };
  interface Both {
    child: ReactNode;
  }
  interface Uses extends Both {
    label: string;
  }
  const node: Uses = { label: "x", child: null };
  return node.child ?? null;
}
