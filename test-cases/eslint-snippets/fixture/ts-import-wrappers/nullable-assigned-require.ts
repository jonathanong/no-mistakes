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
