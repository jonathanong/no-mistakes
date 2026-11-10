import { readdirSync } from "node:fs";
import { join } from "node:path";

export function fn(src: string, dest: string) {
  for (const entry of readdirSync(src, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      fn(join(src, entry.name), join(dest, entry.name));
      continue;
    }
  }
}
