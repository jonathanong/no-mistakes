const { parseDate: readDate } = require("./utils.mts");

export const requiredAliasDate = "2026-01-01";

function renderRequiredAliasDate(input: string) {
  return readDate(input).toISOString();
}

export const renderedRequiredAliasDate = renderRequiredAliasDate(requiredAliasDate);
