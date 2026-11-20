export async function load(extra: unknown) {
  const parts = await import(
    // require("./orders/items")
    "./widgetColumns"
  );
  parts.append(extra);
}
