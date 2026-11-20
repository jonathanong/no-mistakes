import * as columns from "./renameStar";

export function extend(extra: unknown) {
  columns["widget" + "Columns"].append(extra);
}
