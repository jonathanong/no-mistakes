import { readdirSync } from "node:fs";
import { join } from "node:path";

export function scan(): string[] {
  const dir = process.env.EXAMPLE_LOCAL_DIR ?? join(process.cwd(), "tmp", "example");
  return readdirSync(dir);
}
