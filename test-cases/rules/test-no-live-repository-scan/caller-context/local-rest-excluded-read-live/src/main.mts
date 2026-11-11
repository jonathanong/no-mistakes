import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
export function main(options) { const { root, ...rest } = options; return readdirSync(rest.root ?? process.cwd()); }
