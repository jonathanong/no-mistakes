import { readdirSync } from "node:fs";

export function scan(root: string) {
  return readdirSync(root);
}
