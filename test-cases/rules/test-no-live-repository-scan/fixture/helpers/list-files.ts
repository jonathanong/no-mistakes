import { readdirSync } from "node:fs";

export function listFiles(dir: string): string[] {
  return readdirSync(dir);
}
