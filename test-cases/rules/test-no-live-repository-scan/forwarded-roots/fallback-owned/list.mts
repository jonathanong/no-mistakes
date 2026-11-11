import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
const project = { root: "src", test: { root: "tests" } };
export function list(root: string) { return readdirSync(resolve(root, project.root ?? project.test?.root ?? ".")); }
