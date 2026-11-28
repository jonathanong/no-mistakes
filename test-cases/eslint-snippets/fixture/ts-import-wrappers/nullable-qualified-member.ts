declare const moduleName: string;

export async function qualified(): Promise<Widget | null> {
  const { api: client } = require(moduleName);
  return client.getWidget("1");
}

export async function otherMethod(): Promise<Widget | null> {
  const { api: client } = require(moduleName);
  return client.other("1");
}

export async function inlineChain(): Promise<Widget | null> {
  return require("./data").api.getWidget("1");
}

export async function inlineDynamic(): Promise<Widget | null> {
  return require(moduleName).api.getWidget("1");
}

const getWidget = require("./data").api.getWidget;

export async function sameName(): Promise<Widget | null> {
  return getWidget("1");
}

const group = require("./data").api;

export async function splitGroup(): Promise<Widget | null> {
  const read = group.getWidget;
  return read("1");
}

const svc = require("./data").services.api;

export async function servicePrefix(): Promise<Widget | null> {
  return svc.getWidget("1");
}
