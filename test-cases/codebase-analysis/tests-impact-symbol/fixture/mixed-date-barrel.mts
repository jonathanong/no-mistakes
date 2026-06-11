import { parseDate } from "./utils.mts";

export { parseDate };

function renderDate(input: string) {
  return parseDate(input).toISOString();
}

export const renderedDate = renderDate("2026-01-01");
