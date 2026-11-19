import * as ns from "./assignControls";

export function extend(extra: unknown) {
  Object.assign(ns, { extra });
}
