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

import * as React from "react";
const memberRun = React.use;
const memberBridge = memberRun;
export function memberShadow() {
  const memberRun = memberBridge;
  return memberRun(Promise.resolve(1));
}

let rebound = React.use;
export function letAlias() {
  return rebound(Promise.resolve(1));
}

var varRebound = React.use;
export function varAlias() {
  return varRebound(Promise.resolve(1));
}

let Required = require("react");
export function letRequire() {
  return Required.use(Promise.resolve(1));
}

let Written = require("react");
Written = null;
export function writtenLetRequire() {
  return Written.use(Promise.resolve(1));
}

let local = (value: unknown) => value;
local = (value: unknown) => value;
export function localRewrite() {
  return local(Promise.resolve(1));
}

let adopted = (value: unknown) => value;
adopted = use;
export function adoptedUse() {
  return adopted(Promise.resolve(1));
}

declare const method: string;
declare const moduleName: string;
const dynamicReact = require(moduleName);
export function dynamicModule() {
  return dynamicReact[method](Promise.resolve(1));
}
export function dynamicReactKey() {
  return React[method](Promise.resolve(1));
}
const localApi = {
  use(value: unknown) {
    return value;
  },
};
export function localDynamic() {
  return localApi[method](Promise.resolve(1));
}
