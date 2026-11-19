import * as ns from "./namespaceElementControls";

export function extend(dynamicFragment: unknown) {
  ns["NS_ELEMENT_COLS"].append(dynamicFragment);
}
