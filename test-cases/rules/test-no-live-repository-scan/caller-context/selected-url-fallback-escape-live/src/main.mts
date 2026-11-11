import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
function mutate(value) { if (value instanceof URL) value.pathname = process.cwd(); }
export function main(options) { const selected = options.nested.root ?? pathToFileURL(mkdtempSync("/tmp/fallback-")); mutate(selected); return readdirSync(selected); }
