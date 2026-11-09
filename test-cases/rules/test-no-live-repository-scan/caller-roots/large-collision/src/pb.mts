import { readdirSync } from "node:fs";
import { join } from "node:path";

export function pb(root: string) {
  return readdirSync(join(root, "x"));
}
