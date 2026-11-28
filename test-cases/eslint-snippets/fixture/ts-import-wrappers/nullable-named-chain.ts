import * as data from "./data";
import { api } from "./data";

export async function namespaceChain(): Promise<Widget | null> {
  return data.api.getWidget("1");
}

export async function namedDestructure(): Promise<Widget | null> {
  const { getWidget: read } = api;
  return read("1");
}
