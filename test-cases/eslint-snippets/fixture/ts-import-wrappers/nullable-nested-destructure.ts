declare const id: string;
declare const missing: (id: string) => Promise<Widget | null>;

export async function nested(): Promise<Widget | null> {
  const {
    api: { getWidget: read },
  } = require("./data");
  return read(id);
}

export async function defaulted(): Promise<Widget | null> {
  const {
    api: { getWidget: read = missing },
  } = require("./data");
  return read(id);
}

export async function dynamicNested(): Promise<Widget | null> {
  const {
    [id]: { getWidget: read },
  } = require("./data");
  return read(id);
}

export async function defaultedObject(): Promise<Widget | null> {
  const { api: { getWidget: read } = { getWidget: missing } } = require("./data");
  return read(id);
}

declare const localApi: { getWidget(id: string): Promise<Widget | null> };

export async function ancestorLocal(): Promise<Widget | null> {
  const { api: { getWidget: read } = localApi } = require("./data");
  return read(id);
}

export async function ancestorSame(): Promise<Widget | null> {
  const { api: { getWidget: read } = require("./data").api } = require("./data");
  return read(id);
}

export async function ancestorOther(): Promise<Widget | null> {
  const { api: { getWidget: read } = require("./data").other } = require("./data");
  return read(id);
}
