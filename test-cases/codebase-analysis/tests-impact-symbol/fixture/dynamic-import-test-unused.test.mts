export async function testDynamicImportWithoutParseDate() {
  const dates = await import("./utils.mts");
  dates.formatDate(new Date("2026-01-01"));
}
