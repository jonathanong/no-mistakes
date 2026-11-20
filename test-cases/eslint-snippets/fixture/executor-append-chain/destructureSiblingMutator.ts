import * as parts from "./destructureSibling";

const { SIBLING_COLS } = parts;

export function extend(dynamicFragment: unknown) {
  SIBLING_COLS.append(dynamicFragment);
}
