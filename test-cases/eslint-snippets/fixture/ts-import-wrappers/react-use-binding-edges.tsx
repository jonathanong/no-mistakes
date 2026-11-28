import { use } from "react";

declare const local: (value: Promise<number>) => number;

export function defaultedRead(value: Promise<number>) {
  const { missing: read = use } = require("./empty.cjs");
  return read(Promise.resolve(value));
}

export function localDefault(value: Promise<number>) {
  const { use: read = local } = require("react");
  return read(Promise.resolve(value));
}

export function sameDefault(value: Promise<number>) {
  const { use: read = use } = require("react");
  return read(Promise.resolve(value));
}

export function typeShadow(value: Promise<number>) {
  const R = require("react");
  {
    type R = string;
    return R.use(Promise.resolve(value));
  }
}

export function repeatedVar(value: Promise<number>) {
  var R;
  var R;
  R = require("react");
  return R.use(Promise.resolve(value));
}

export function repeatedLocal(value: Promise<number>) {
  var read;
  var read;
  read = local;
  return read(Promise.resolve(value));
}

export function assignedAfter(value: Promise<number>) {
  let R;
  const result = R?.use(Promise.resolve(value));
  R = require("react");
  return result;
}

export function assignedBefore(value: Promise<number>) {
  let R;
  R = require("react");
  return R.use(Promise.resolve(value));
}

const useKey = "use";
export function constDestructure(value: Promise<number>) {
  const { [useKey]: read } = require("react");
  return read(Promise.resolve(value));
}

declare const dynamicKey: string;
export function dynamicDestructureKey(value: Promise<number>) {
  const { [dynamicKey]: read } = require("react");
  return read(Promise.resolve(value));
}
