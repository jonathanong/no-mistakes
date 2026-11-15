import { readdirSync } from "node:fs";

export function main({ outputRoot }: { outputRoot: string }) {
  return readdirSync(outputRoot);
}
