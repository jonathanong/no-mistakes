import * as api from "./api";

declare const method: string;

export async function dynamic(): Promise<Widget | null> {
  return api[method]("1");
}

export async function literal(): Promise<Widget | null> {
  return api["getWidget"]("1");
}

export async function dotted(): Promise<Widget | null> {
  return api.getWidget("1");
}

declare function load(): { getWidget(id: string): Promise<Widget | null> };

export async function called(): Promise<Widget | null> {
  return load().getWidget("1");
}

const items: Record<string, (id: string) => Promise<Widget | null>> = {};

export async function local(): Promise<Widget | null> {
  return items[method]("1");
}
