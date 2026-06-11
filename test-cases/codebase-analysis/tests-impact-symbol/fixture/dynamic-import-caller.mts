export async function loadDateDynamically(input: string) {
  const dates = await import("./utils.mts");
  return dates.parseDate(input);
}
