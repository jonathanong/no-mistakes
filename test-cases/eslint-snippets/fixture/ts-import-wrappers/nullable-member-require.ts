export async function member(): Promise<Widget | null> {
  return require("./w").getWidget("1");
}
