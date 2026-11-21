import * as columns from "./defaultBarrel";

export function extend(extra: unknown) {
  columns.default.append(extra);
  columns.keptColumns.append(extra);
}
