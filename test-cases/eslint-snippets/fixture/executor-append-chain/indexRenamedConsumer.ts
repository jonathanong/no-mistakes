import { Final } from "./indexRenamedMutator";

export function extend(dynamicFragment: unknown) {
  Final.append(dynamicFragment);
}
