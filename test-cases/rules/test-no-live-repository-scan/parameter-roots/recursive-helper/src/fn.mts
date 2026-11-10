import { readdirSync } from "node:fs";
import { join } from "node:path";

export async function snapshotFiles(root: string) {
  function visit(dir: string) {
    for (const child of readdirSync(dir)) {
      visit(join(dir, child));
    }
  }
  visit(root);
}
