import { readdirSync } from "node:fs";
import { join } from "node:path";

export function assetEscape(name: string) {
  return readdirSync(join(import.meta.dirname, "..", "assets", "migrations", name));
}
