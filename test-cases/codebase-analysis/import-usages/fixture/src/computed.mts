export async function load(moduleName: string) {
  const dynamic = await import(moduleName);
  const required = require(moduleName);
  return { dynamic, required, local: require("./local.cjs") };
}
