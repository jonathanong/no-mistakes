import { readdirSync } from "node:fs";

export function sync(root: string) {
  return readdirSync(root);
}

const marker = "scan";
export { marker };
export * from "./loop.mts";
