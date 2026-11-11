import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
export function main(first, second, project) { return readdirSync(resolve(join(first, "a") ?? join(second, "b"), project.root)); }
