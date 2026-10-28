import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const root = join(process.cwd(), ".github", "workflows");

export function allJobEntries(): string[] {
  return readdirSync(root).map((name) => readFileSync(join(root, name), "utf8"));
}
