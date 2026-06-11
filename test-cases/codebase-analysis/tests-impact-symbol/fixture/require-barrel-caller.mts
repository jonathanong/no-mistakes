const dates = require("./date-barrel.mts");

export const requiredBarrelDate = "2026-01-01";

function renderRequiredBarrelDate(input: string) {
  return dates.parseDate(input).toISOString();
}

export const renderedRequiredBarrelDate = renderRequiredBarrelDate(requiredBarrelDate);
