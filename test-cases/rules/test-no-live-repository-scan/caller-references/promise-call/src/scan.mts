import { readdirSync } from "node:fs";

export function scan(root: string) {
  return readdirSync(root);
}

export function forward(root: string) {
  return Promise.resolve(root).call(scan);
}
