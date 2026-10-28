import { readdirSync } from "node:fs";
import { join } from "node:path";

export function policyNames(): string[] {
  return readdirSync(join(process.cwd(), ".github"));
}
