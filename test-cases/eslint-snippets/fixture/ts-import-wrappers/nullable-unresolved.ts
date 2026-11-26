declare const widgetModule: string;
const { getWidget: load } = require(widgetModule);
export async function hidden(): Promise<Widget | null> {
  return load("1");
}
