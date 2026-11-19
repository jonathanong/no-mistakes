import DEFAULTED from "./aliasReexport";
import { RENAMED } from "./aliasReexport";

export function extend(dynamicFragment: unknown) {
  RENAMED.append(dynamicFragment);
  DEFAULTED.append(dynamicFragment);
}
