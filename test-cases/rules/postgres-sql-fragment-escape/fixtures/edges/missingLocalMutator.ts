import * as items from "./missing-widget";

export function extend(extra: unknown) {
  Object.assign(items, { extra });
}
