import { readdirSync } from "node:fs";

export function scan(): string[] {
  const captured = process.env.EXAMPLE_LOCAL_DIR ?? "./tmp/example";
  return readdirSync(captured);
}
