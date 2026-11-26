import type { ReactNode } from "react";
import R = require("react");

function A(x: R.ReactNode) {
  return x ?? "fallback";
}

function B(x: import("react").ReactNode) {
  return x ?? "fallback";
}

interface Props {
  children: ReactNode;
}

function C(children: ReactNode) {
  return (children satisfies ReactNode) ?? null;
}

function D(props?: Props) {
  return props!.children ?? null;
}

function E(props?: Props) {
  const { children } = props!;
  return children ?? null;
}
