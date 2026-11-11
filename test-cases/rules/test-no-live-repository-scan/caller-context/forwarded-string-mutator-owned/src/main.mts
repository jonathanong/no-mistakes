import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
export function scan(options) { return readdirSync(options.nested.root); }
function mutate(value) { if (value instanceof URL) value.pathname = process.cwd(); }
export function main(options) { mutate(options.nested.root); return scan(options); }
