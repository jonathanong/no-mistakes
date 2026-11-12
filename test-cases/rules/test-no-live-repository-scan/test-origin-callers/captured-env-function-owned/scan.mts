import { readdirSync } from "node:fs";
import { join } from "node:path";
export function validate() {
  const root = process.env.SCAN_ROOT!;
  return readdirSync(join(root, "rules"));
}
