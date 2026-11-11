import { readdirSync } from "node:fs";
export function validateArtifact(root: string, opts?: { maxFiles: number }) {
  const files = readdirSync(root);
  return readdirSync(root).slice(0, opts?.maxFiles ?? files.length);
}
