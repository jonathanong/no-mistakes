declare const reactModule: string;
const R = require(reactModule);

function A(x: R.ReactNode) {
  return x ?? "fallback";
}

declare namespace Other {
  export type ReactNode = string;
}
import React = Other;
function defaultName(value: React.ReactNode) {
  return value ?? "fallback";
}

type Slot = R.ReactNode;
function shadowed() {
  type Slot = string;
  const text: Slot = "";
  return text ?? "";
}
function outerSlot(value: Slot) {
  return value ?? "";
}
type Plain = string;
type Repeated = Plain | Plain;
function repeatedAlias(value: Repeated) {
  return value ?? "";
}
type Left = Right;
type Right = Left;
function aliasCycle(value: Left) {
  return value ?? "";
}
export type ExportedLeft = ExportedRight;
export type ExportedRight = ExportedLeft;
function exportedAliasCycle(value: ExportedLeft) {
  return value ?? "";
}
