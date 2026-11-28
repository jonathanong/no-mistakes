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

export async function destructured(id: string): Promise<Widget | null> {
  const { getWidget: read } = api;
  return read(id);
}

export async function flat(id: string): Promise<Widget | null> {
  const { getWidget: read } = data;
  return read(id);
}

export async function nestedDefault(id: string): Promise<Widget | null> {
  const { default: read } = data;
  return read.getWidget(id);
}

export async function nestedDefaultMember(id: string): Promise<Widget | null> {
  const {
    default: { getWidget: read },
  } = data;
  return read(id);
}

export async function secondDefault(id: string): Promise<Widget | null> {
  const {
    default: { getWidget: read },
  } = api;
  return read(id);
}
