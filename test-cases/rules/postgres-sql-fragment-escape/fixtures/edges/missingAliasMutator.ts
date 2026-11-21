import * as items from "@widgets/missing-item";

export function extend(extra: unknown) {
  Object.assign(items, { extra });
}
