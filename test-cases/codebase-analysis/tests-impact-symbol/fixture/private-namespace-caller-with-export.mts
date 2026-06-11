import * as dates from "./utils.mts";

export const unrelatedNamespaceValue = 1;

function renderNamespaceDate(value: string) {
  return dates.parseDate(value);
}

export const renderedNamespaceDate = renderNamespaceDate("2026-06-11");
