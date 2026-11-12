import { readdirSync } from "node:fs";

export function scan(): string[] {
  return readdirSync(process.env.EXAMPLE_LOCAL_DIR ?? "./tmp/example");
}
