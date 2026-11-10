import { readdirSync } from "node:fs";
import { join } from "node:path";

export function listFiles(root: string) {
  const out: string[] = [];
  function visit(rel: string) {
    for (const entry of readdirSync(join(root, rel), { withFileTypes: true })) {
      out.push(join(rel, entry.name));
      visit(join(rel, entry.name));
    }
  }
  visit("");
  return out;
}
