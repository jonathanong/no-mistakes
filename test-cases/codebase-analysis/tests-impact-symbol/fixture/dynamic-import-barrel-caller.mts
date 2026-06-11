export async function renderDynamicBarrelDate(input: string) {
  const dates = await import("./date-barrel.mts");
  return dates.parseDate(input).toISOString();
}
