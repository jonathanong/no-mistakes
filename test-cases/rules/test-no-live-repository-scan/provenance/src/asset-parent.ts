import { readdirSync } from "node:fs";
import { join } from "node:path";

export function assetParent() {
  return readdirSync(join(import.meta.dirname, "..", "assets"));
}
