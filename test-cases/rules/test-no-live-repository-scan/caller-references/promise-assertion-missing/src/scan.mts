import { readdirSync } from "node:fs";

export function scan(root: string) {
  return readdirSync(root);
}

export function assertOwned(root: string) {
  return Promise.resolve(root).then((directory) => scan(directory));
}
