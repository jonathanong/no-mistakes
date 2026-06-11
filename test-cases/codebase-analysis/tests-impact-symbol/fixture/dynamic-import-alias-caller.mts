async function renderDynamicAliasDate(input: string) {
  const { parseDate: pd } = await import("./utils.mts");
  return pd(input).toISOString();
}

export const renderedDynamicAliasDate = await renderDynamicAliasDate("2026-01-01");
