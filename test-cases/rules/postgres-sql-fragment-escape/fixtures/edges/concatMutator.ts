import * as columns from "./widgetColumns";

export function extend(extra: unknown) {
  columns["widget" + "Columns"].append(extra);
}
