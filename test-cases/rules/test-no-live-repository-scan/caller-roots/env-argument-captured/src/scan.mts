import { readdirSync } from "node:fs";
import { join } from "node:path";

export function scanBefore(root: string) {
  return readdirSync(join(root, "items"));
}

export function scanAfter(root: string) {
  return readdirSync(join(root, "items"));
}
