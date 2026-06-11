import { parse } from "./aliased-local-date-barrel.mts";

export const unrelatedAliasedBarrelValue = 1;

function renderAliasedBarrelDate(value: string) {
  return parse(value);
}

export const renderedAliasedBarrelDate = renderAliasedBarrelDate("2026-06-11");
