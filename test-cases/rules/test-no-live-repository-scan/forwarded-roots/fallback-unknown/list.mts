import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(root: string) { return readdirSync(resolve(root, project.root ?? project.test?.root ?? ".")); }
