import { readdirSync } from "node:fs";
import { join } from "node:path";

export function migrationFiles() {
  return readdirSync(join(import.meta.dirname, "..", "assets", "migrations"));
}
