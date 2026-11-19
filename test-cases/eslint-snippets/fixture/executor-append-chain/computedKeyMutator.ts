import * as ns from "./computedKeyControls";

export function extend(extra: unknown) {
  const key: string = "KEY_A";
  ns[key].append(extra);
}
