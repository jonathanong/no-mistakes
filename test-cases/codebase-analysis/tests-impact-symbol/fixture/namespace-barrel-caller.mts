import { dates } from "./namespace-date-barrel.mts";

export const namespaceBarrelDate = "2026-01-01";

function renderNamespaceBarrelDate(input: string) {
  return dates.parseDate(input).toISOString();
}

export const renderedNamespaceBarrelDate = renderNamespaceBarrelDate(namespaceBarrelDate);
