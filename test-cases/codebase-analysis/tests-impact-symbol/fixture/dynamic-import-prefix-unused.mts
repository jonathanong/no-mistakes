export async function renderOldDynamicDate() {
  const utils = await import("./utils.mts");
  return utils.parseDateOld("2026-01-01");
}
