export async function renderWorkspaceDynamicDate(input: string) {
  const dates = await import("@repo/dates");
  return dates.parseDate(input).toISOString();
}
