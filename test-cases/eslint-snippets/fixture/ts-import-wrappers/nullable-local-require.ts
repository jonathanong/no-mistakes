function require(name: string): { getWidget(id: string): Promise<Widget | null> } {
  return { getWidget: () => Promise.resolve(null) };
}
export async function local(): Promise<Widget | null> {
  return require("./w").getWidget("1");
}
