import { readdirSync } from "node:fs";
import { join } from "node:path";

export function nearMiss() {
  return readdirSync(join(import.meta.dirname, "..", "assets", "migrations-src"));
}
