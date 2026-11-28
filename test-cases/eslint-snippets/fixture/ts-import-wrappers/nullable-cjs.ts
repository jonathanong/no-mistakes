const { getWidget: load } = require("./w");
export async function a(): Promise<Widget | null> {
  return load("1");
}

const w = require("./w");
export async function b(): Promise<Widget | null> {
  return w.getWidget("1");
}

import eq = require("./w");
export async function c(): Promise<Widget | null> {
  return eq.getWidget("1");
}

import { getWidget } from "./w";
const { wrapNullable: wn } = require("./wrap");
export async function d(): Promise<Widget | null> {
  return wn(getWidget("1"));
}

import * as wrap from "./wrap";
export async function e(): Promise<Widget | null> {
  return wrap.wrapNullable(getWidget("1"));
}
