import { readdirSync } from "node:fs";
import { join } from "node:path";

// Climbs out of whatever root the caller owns, so this is a live scan.
export function listTwoUp(dir: string) {
  return readdirSync(join(dir, "..", ".."));
}
