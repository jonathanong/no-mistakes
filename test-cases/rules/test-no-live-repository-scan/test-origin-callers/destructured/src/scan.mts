import { readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
export function check({ root, paths }: { root: string; paths: string[] }) {
  return paths.map(path => readdirSync(join(root, path)));
}
export function production() {
  return check({ root: fileURLToPath(new URL("..", import.meta.url)), paths: ["src"] });
}
