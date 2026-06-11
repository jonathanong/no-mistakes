const { parseDate } = require("./utils.mts");

export const requiredDate = "2026-01-01";

function renderRequiredDate(input: string) {
  return parseDate(input).toISOString();
}

export const renderedRequiredDate = renderRequiredDate(requiredDate);
