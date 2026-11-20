import * as columns from "./renameStar";

export function extend(extra: unknown) {
  columns.widgetColumns.append(extra);
  columns.default.append(extra);
  columns.keptColumns.append(extra);
}
