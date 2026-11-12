import { readdirSync } from "node:fs";
import { join } from "node:path";
export function validate({ root, paths }: { root: string; paths: string[] }) { return paths.flatMap(path => readdirSync(join(root, path))); }
