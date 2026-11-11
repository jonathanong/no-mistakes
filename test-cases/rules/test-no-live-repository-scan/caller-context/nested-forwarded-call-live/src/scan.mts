import { readdirSync } from "node:fs";
export function scan(options) {
  return readdirSync(options.nested.root);
}
