import { readdirSync } from "node:fs";

export function main(argv: string[]) {
  const index = argv.indexOf("--output-root");
  return readdirSync(argv[index + 1]!);
}
