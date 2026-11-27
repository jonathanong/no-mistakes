declare const apiModule: string;
const api = require(apiModule);

export async function direct(): Promise<Widget | null> {
  return api.getWidget("1");
}

export async function forwarded(): Promise<Widget | null> {
  return api.getWidget.call(null, "1");
}

export async function listed(): Promise<Widget | null> {
  return api.list.call(null, "1");
}
