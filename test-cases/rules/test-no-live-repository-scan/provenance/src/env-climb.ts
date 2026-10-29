import { readdirSync } from "node:fs";
import { join } from "node:path";

const getDir = () => process.env.CLIMB_DIR ?? "./tmp/climb";

// Climbs out of the owned root with `..`, so this is a live scan.
export function listAbove() {
  return readdirSync(join(getDir(), "..", "src"));
}
