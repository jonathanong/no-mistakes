import * as ns from "./namespaceDestructureControls";

const { NS_DESTRUCTURED_COLS } = ns;
const key = "unrelated";
const { [key]: computed, missing } = {} as Record<string, unknown>;

export function extend(dynamicFragment: unknown) {
  NS_DESTRUCTURED_COLS.append(dynamicFragment);
  return [computed, missing];
}
