import { globSync } from "node:fs";
import { resolve } from "node:path";
export function count(worktreeRoot, project) {
  const projectRoot = resolve(worktreeRoot, project.root ?? project.test?.root ?? ".");
  return globSync("**/*", { cwd: projectRoot });
}
