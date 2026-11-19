import ext = require("./equalsControls");

export function extend(extra: unknown) {
  ext.EQUALS_COLS.append(extra);
}
