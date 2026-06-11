import { parseDate } from "./date-barrel.mts";

export const unrelatedBarrelValue = 1;

function renderBarrelDate(value: string) {
  return parseDate(value);
}

export const renderedBarrelDate = renderBarrelDate("2026-06-11");
