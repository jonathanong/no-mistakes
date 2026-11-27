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
