declare const moduleName: string;
const wrapNullable = require(moduleName);
import { getWidget } from "./api";

export function read(): Promise<Widget | null> {
  return wrapNullable(getWidget());
}

const api = require(moduleName);

export function member(): Promise<Widget | null> {
  return api.wrapNullable(getWidget());
}

const resolved = require("./wrap");

export function kept(): Promise<Widget | null> {
  return resolved.wrapNullable(getWidget());
}

export function destructured(): Promise<Widget | null> {
  const { wrapNullable } = require(moduleName);
  return wrapNullable(getWidget());
}

export function thisArg(): Promise<Widget | null> {
  return wrapNullable.call(getWidget());
}

export function appliedThis(): Promise<Widget | null> {
  return wrapNullable.apply(getWidget(), []);
}

export function forwardedArg(): Promise<Widget | null> {
  return wrapNullable.call(null, getWidget());
}

const rpc = require(moduleName);

export function unresolvedReceiver(): Promise<Widget | null> {
  return rpc.call(getWidget());
}

function relay(value: Promise<Widget | null>) {
  return value;
}

export function resolvedReceiver(): Promise<Widget | null> {
  return relay.call(getWidget());
}
