import { readdirSync } from "node:fs";
import { join } from "node:path";

export function fn(repoRoot: string) {
  readdirSync(join(repoRoot, ".x/a"), { recursive: true });
  readdirSync(join(repoRoot, ".x/b"), { recursive: true });
}
