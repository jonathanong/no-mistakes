import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
export function main(root, project) { return readdirSync(resolve(root, project.root ?? project.test?.root ?? ".")); }
