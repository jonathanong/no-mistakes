import { LISTED_COLS } from "./listedConstControls";

export function extend(dynamicFragment: unknown) {
  LISTED_COLS.append(dynamicFragment);
}
