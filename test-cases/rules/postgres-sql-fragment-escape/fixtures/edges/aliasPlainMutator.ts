import * as widgets from "@widgets/plain";

export function extend(extra: unknown) {
  Object.assign(widgets, { extra });
}
