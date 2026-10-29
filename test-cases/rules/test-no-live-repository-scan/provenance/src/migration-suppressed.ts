import { readdirSync } from "node:fs";
import { join } from "node:path";

export function suppressedMigrations() {
  // no-mistakes-disable-next-line test-no-live-repository-scan -- reviewed shipped asset
  return readdirSync(join(import.meta.dirname, "..", "assets", "migrations"));
}
