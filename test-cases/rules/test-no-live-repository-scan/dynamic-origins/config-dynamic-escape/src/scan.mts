import { readdirSync } from "node:fs";

export function runSmoke(libraryDirectory = process.env.TMPDIR) {
  return readdirSync(libraryDirectory);
}
