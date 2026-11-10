import { readdirSync } from "node:fs";
import { join } from "node:path";

export function enumerate(root: string) {
  readdirSync(join(root, ".x/a"), { recursive: true });
  readdirSync(join(root, ".x/b"), { recursive: true });
}

export function other(root: string) {
  readdirSync(join(root, ".x/a"), { recursive: true });
}
