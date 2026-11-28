import * as data from "./data";
import read from "./data";

const api = data.default;

export async function aliased(id: string): Promise<Widget | null> {
  return api.getWidget(id);
}

export async function direct(id: string): Promise<Widget | null> {
  return data.default.getWidget(id);
}

export async function ordinary(id: string): Promise<Widget | null> {
  return read.getWidget(id);
}
