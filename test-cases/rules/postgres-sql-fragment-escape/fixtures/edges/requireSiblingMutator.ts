const parts = {},
  other = require("./widgetColumns");

export function extend(extra: unknown) {
  other.append(extra);
}
