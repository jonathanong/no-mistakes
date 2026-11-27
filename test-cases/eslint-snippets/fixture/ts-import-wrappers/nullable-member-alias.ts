declare const key: string;
const read = require("./data").getWidget;
const missing = require("./data")[key];

export async function named(): Promise<Widget | null> {
  return read("1");
}

export async function hidden(): Promise<Widget | null> {
  return missing("1");
}

import * as api from "./data";
const namespaced = api.getWidget;
const hiddenName = api[key];

export async function fromNamespace(): Promise<Widget | null> {
  return namespaced("1");
}

export async function hiddenNamespace(): Promise<Widget | null> {
  return hiddenName("1");
}
