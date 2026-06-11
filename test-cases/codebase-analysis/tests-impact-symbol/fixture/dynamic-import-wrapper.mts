export async function loadWrappedDate(input: string) {
  const utils = await import("./utils.mts");
  return utils.parseDate(input);
}
