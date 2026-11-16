import { readdirSync } from "node:fs";

export function processSync(root: string) {
  return readdirSync(root);
}

export const workers = { sync: processSync };
export const marker = true;
