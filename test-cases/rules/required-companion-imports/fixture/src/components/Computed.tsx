export async function load(moduleName: string) {
  return import(moduleName);
}
