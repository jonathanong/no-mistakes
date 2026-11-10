import { readdirSync } from "node:fs";
import { join } from "node:path";

export const RESULT_PREFIX = "results-";

export function fn(repoRoot: string) {
  readdirSync(join(repoRoot, ".x/a"), { recursive: true });
  readdirSync(join(repoRoot, ".x/b"), { recursive: true });
}
