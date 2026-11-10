import { readdirSync } from "node:fs";

export function listFiles(root: string) {
  readdirSync(root);
}
