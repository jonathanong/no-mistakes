export async function renderDynamicOtherExportName(input: string) {
  const utils = await import("./utils.mts");
  return utils.parse(input);
}
