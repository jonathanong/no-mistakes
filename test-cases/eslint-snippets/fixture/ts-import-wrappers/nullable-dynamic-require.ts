declare const moduleName: string;
declare const key: string;
export async function dynamicModule(): Promise<Widget | null> {
  return require(moduleName).getWidget("1");
}
export async function dynamicMember(): Promise<Widget | null> {
  return require("./w")[key]("1");
}
