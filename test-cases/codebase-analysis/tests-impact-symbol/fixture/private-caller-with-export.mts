import { parseDate } from "./utils.mts";

export const unrelatedDateValue = 1;

function renderDate(value: string) {
  return parseDate(value);
}

export const renderedDate = renderDate("2026-06-11");
