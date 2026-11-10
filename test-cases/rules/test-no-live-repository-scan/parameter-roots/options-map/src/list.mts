import { readdirSync } from "node:fs";
import { join } from "node:path";

export function list(options: { outputRoot: string }) {
  return walk(options.outputRoot, options.outputRoot);
}

function walk(dir: string, root: string): string[] {
  return readdirSync(join(dir, root), { withFileTypes: true }).map((entry) => {
    const next = join(dir, entry.name);
    return entry.isDirectory() ? walk(next, root).join("\n") : next;
  });
}
