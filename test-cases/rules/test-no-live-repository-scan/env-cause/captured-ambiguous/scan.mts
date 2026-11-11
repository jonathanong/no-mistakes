import { readdirSync } from "node:fs";

const captured = (unknownRoot || process.env.FIRST) || process.env.SECOND;
export function scan(): string[] {
  return readdirSync(captured);
}
