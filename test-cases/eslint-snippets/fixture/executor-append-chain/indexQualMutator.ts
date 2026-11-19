import * as qualified from "./indexQualFragments";

import Qualified = qualified.QUAL_COLS;

export function extend(dynamicFragment: unknown) {
  Qualified.append(dynamicFragment);
}
