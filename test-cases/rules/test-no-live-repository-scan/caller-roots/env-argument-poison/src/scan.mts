import { readdirSync } from "node:fs";
import { join } from "node:path";

export function scan(root: string) {
  return readdirSync(join(root, "items"));
}
