declare const moduleName: string;

export async function alternation(): Promise<Widget | null> {
  const service = require(moduleName);
  return service.getWidget("1");
}

export async function otherMember(): Promise<Widget | null> {
  const service = require(moduleName);
  return service.other("1");
}
