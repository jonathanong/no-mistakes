export async function loadUnusedDateModuleWithShadowedMember(input: string) {
  const dates = await import("./utils.mts");
  const other = {
    parseDate(value: string) {
      return new Date(value);
    },
  };
  return other.parseDate(input).toISOString() + String(Boolean(dates));
}
