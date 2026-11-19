import * as ns from "./elementControls";

export function extend(extra: unknown) {
  ns["ELEMENT_MUT"].append(extra);
}
