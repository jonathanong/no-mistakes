import { parseDate } from "./utils.mts";

export const unrelatedTestValue = 1;

function expectParsedDate(value: string) {
  return parseDate(value);
}

expectParsedDate("2026-06-11");
