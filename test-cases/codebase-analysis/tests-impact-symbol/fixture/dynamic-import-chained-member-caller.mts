export async function renderDynamicChainedDate(input: string) {
  return (await import("./utils.mts")).parseDate(input).toISOString();
}
