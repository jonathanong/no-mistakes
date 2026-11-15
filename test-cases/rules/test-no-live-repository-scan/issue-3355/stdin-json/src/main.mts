import { readdirSync } from "node:fs";

export function main(raw: string) {
  const input = JSON.parse(raw) as { outputRoot: string };
  return readdirSync(input.outputRoot);
}
