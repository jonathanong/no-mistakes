import * as ns from "./reflectControls";

export function extend(extra: unknown) {
  Reflect.set(ns, "label", extra);
  return Reflect.get(ns, "label");
}
