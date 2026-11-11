import { readdirSync } from "node:fs";
export function main(paths) {
  return readdirSync(paths.schemaDir || process.cwd());
}
