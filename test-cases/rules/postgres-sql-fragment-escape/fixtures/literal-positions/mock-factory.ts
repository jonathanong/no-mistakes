vi.mock<typeof import("./columns")>(import("./columns"), async (importOriginal) => {
  const actual = await importOriginal();
  const { widgetColumns } = await import("./columns");
  widgetColumns.append(1);
  return { ...actual, widgetColumns };
});
