import { readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
export function scanArtifact(root: string) {
  const names = readdirSync(root);
  if (names.includes("leak")) throw new Error("Unsafe");
  if (names.includes("a") || names.includes("b")) throw new Error("x");
  return names;
}
export function writeArtifact(root: string) {
  writeFileSync(join(root, "index.html"), "safe");
}
