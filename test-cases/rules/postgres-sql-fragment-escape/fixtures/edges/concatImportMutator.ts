declare const suffix: string;

export async function extend(extra: unknown) {
  const parts = await import("./widgetColumns" + suffix);
  parts.append(extra);
}
