import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
export function main(options) { options.root = process.cwd(); const { ignored, ...rest } = options; return readdirSync(rest.root ?? process.cwd()); }
