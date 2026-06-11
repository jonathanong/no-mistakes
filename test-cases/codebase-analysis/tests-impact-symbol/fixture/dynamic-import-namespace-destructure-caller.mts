export async function renderDynamicNamespaceDestructureDate(input: string) {
  const { dates } = await import("./namespace-date-barrel.mts");
  return dates.parseDate(input).toISOString();
}
