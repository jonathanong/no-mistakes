import { readdirSync } from "node:fs";
import { join } from "node:path";

export function scan(root: string) {
  return readdirSync(join(root, "items"));
}

export function forward(root: string) {
  return scan(root);
}

export function scanUnowned(root: string) {
  return readdirSync(join(root, "items"));
}

export function scanLate(root: string) {
  return readdirSync(join(root, "items"));
}
