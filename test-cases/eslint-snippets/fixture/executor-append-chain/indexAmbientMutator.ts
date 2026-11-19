import { VIRTUAL_COLS } from "ambient-cols";

export function extend(dynamicFragment: unknown) {
  VIRTUAL_COLS.append(dynamicFragment);
}
