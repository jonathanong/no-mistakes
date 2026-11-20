export async function extend(extra: unknown) {
  const parts = await import("./widgetColumns");
  parts.append(extra);
}
