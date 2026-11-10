import { readdirSync } from "node:fs";
import { join, resolve } from "node:path";

const DEFAULT = ["manifest.json"];
const MODULE_CONST_ROOT = join(import.meta.dirname, "..", "fixtures");

export function exported(source: string, extra = DEFAULT) {
  void extra;
  const root = resolve(source);
  for (const _file of walk(root)) {
    /* walk owns the enumeration */
  }
}

function walk(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? walk(join(dir, entry.name)) : [],
  );
}

export function ensure({
  output,
  source = join(MODULE_CONST_ROOT, "c"),
} = {}) {
  void output;
  return exported(source);
}
