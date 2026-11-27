import { default as R } from "react";
export function defaults() {
  return R.use(Promise.resolve(1));
}

const key = "read";
const { [key]: read } = { read: (value: unknown) => value };
export function localComputed() {
  return read(Promise.resolve(1));
}

let { use: swapped } = require("react");
swapped = (value: unknown) => value;
export function reassigned() {
  return swapped(Promise.resolve(1));
}

import { use as run } from "react";
const bridge = run;
export function shadowed() {
  const run = bridge;
  return run(Promise.resolve(1));
}

import { use } from "react";
export function overload() {
  function use(value: number): number;
  function use(value: string): string;
  function use(value: number | string) {
    return value;
  }
  return use(Promise.resolve(1));
}
export function outerUse() {
  return use(Promise.resolve(1));
}
