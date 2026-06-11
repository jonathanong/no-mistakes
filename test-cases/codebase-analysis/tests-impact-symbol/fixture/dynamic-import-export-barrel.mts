export { parseDate } from "./utils.mts";

export async function renderDynamicExportBarrelDate(input: string) {
  const utils = await import("./utils.mts");
  return utils.parseDate(input).toISOString();
}
