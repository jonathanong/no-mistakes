import { parseDate } from "./utils.mts";

function renderExcludedDate(input: string) {
  return parseDate(input).toISOString();
}

export const excludedRenderedDate = renderExcludedDate("2026-01-01");
