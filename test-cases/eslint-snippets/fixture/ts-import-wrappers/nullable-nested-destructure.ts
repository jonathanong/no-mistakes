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
