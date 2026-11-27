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
