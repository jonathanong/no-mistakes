async function renderUnusedDynamicAlias(input: string) {
  const { parseDate: pd } = await import("./utils.mts");
  return otherpd(input);
}

function otherpd(input: string) {
  return input;
}

export const unusedDynamicAliasDate = await renderUnusedDynamicAlias("2026-01-01");
