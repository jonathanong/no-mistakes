import { readdirSync } from "node:fs";

export function sourceTree() {
  return readdirSync("./src");
}
