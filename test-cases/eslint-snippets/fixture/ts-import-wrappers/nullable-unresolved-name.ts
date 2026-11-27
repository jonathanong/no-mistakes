declare const moduleName: string;
const { getWidget } = require(moduleName);

export async function load(): Promise<Widget | null> {
  return getWidget("1");
}
