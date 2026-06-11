export async function renderDynamicAliasedBarrelDate(input: string) {
  const dates = await import("./aliased-local-date-barrel.mts");
  return dates.parse(input).toISOString();
}
