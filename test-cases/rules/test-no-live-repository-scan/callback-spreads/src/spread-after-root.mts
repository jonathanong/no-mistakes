import { readdirSync } from "node:fs";
export function scanRepository(root: string) {
  readdirSync(root);
}
