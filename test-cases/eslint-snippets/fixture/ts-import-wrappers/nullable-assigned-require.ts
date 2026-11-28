type require = unknown;

export async function typeAlias(): Promise<Widget | null> {
  const { getWidget: read } = require("./data");
  return read();
}

export async function typeParameter<require>(): Promise<Widget | null> {
  const { getWidget: read } = require("./data");
  return read();
}

export async function assigned(): Promise<Widget | null> {
  let read;
  read = require("./data").getWidget;
  return read();
}

export async function unrelated(): Promise<Widget | null> {
  let read;
  read = () => Promise.resolve(null);
  return read();
}

declare const moduleName: string;

export async function assignedTwice(): Promise<Widget | null> {
  let read;
  read = require("./data").getWidget;
  read = require(moduleName).getWidget;
  return read();
}

export async function assignedDynamic(): Promise<Widget | null> {
  let read;
  read = require(moduleName).getWidget;
  return read();
}

export async function destructured(): Promise<Widget | null> {
  let { read } = { read: () => null };
  read = require("./data").getWidget;
  return read();
}

export async function destructuredLocal(): Promise<Widget | null> {
  let { read } = { read: () => null };
  return read();
}

export async function destructuredTwice(): Promise<Widget | null> {
  let { read } = { read: () => null };
  read = require("./data").getWidget;
  read = () => Promise.resolve(null);
  return read();
}

export async function arrayDestructured(): Promise<Widget | null> {
  let [read] = [() => null];
  read = require("./data").getWidget;
  return read();
}

export async function destructuredAlias(): Promise<Widget | null> {
  const api = { read: () => null };
  let { read } = api;
  read = api.read;
  return read();
}

export async function stagedTarget(): Promise<Widget | null> {
  let target;
  let read = target;
  target = require("./data").getWidget;
  read = target;
  return read();
}

export async function patternAssigned(): Promise<Widget | null> {
  let read;
  ({ getWidget: read } = require("./data"));
  return read();
}

export async function patternDynamic(id: string): Promise<Widget | null> {
  let read;
  ({ [id]: read } = require("./data"));
  return read();
}

export async function patternArray(): Promise<Widget | null> {
  let read;
  [read] = require("./data");
  return read();
}

export async function patternLocal(): Promise<Widget | null> {
  let read;
  const local = { getWidget: () => null };
  ({ getWidget: read } = local);
  return read();
}

let settled;
settled = require("./data").getWidget;
export async function afterSettled(): Promise<Widget | null> {
  return settled("1");
}

let deferred;
export async function deferredRead(): Promise<Widget | null> {
  return deferred("1");
}
deferred = require("./data").getWidget;

let plain;
export async function plainLater(): Promise<Widget | null> {
  return plain("1");
}
plain = () => Promise.resolve(null);
