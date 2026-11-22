export async function load(extra: unknown) {
  const parts = await import("./columns");
  parts.append(extra);
}
