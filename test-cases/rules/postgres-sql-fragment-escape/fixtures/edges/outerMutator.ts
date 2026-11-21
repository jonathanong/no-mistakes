import * as columns from "./outerShadow";

export function extend(extra: unknown) {
  columns["order" + "Columns"].append(extra);
}
