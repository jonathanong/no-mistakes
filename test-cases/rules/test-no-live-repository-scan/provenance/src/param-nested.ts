import { readdirSync } from "node:fs";

export function syncNested(dirs: string[]) {
  return dirs.flatMap((dir) => readdirSync(dir));
}
