import { readdirSync } from "node:fs";

export function scanFixture(root: string) {
  return readdirSync(root);
}
