import * as fs from "node:fs";

export function extend(extra: unknown) {
  Object.assign(fs, { extra });
}
