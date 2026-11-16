import { readdirSync } from "node:fs";

export function sync(root: string) {
  return readdirSync(root);
}

export const marker = "scan";
