import { SHARED_COLS } from "./exportedConstControls";

export function extend(dynamicFragment: unknown) {
  SHARED_COLS.append(dynamicFragment);
}
