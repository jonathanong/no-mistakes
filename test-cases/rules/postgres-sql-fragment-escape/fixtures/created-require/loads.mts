import { createRequire } from "node:module";

export function loadEarly() {
  return require("../early-widget.cjs");
}

const require = createRequire(import.meta.url);
const settings = require("../widget-settings.cjs");
const defaults = require("../item-defaults.cjs");

export function loadOrders() {
  const require = createRequire(import.meta.url);
  return require("../order-table.cjs");
}
