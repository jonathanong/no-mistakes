const parts = {},
  other = require("./widgetColumns");

export function extend(extra: unknown) {
  parts.append(extra);
}
