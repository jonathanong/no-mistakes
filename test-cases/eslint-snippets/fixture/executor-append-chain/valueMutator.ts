import * as ns from "./valueControls";

export function extend(extra: unknown) {
  Object.values(ns).forEach((part: any) => part.append(extra));
}
