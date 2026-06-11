export async function loadDateFormatter(input: Date) {
  const dates = await import("./utils.mts");
  return dates.formatDate(input);
}
