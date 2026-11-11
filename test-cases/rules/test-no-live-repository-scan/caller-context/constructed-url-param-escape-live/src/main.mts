import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
function mutate(value) { if (value instanceof URL) value.pathname = process.cwd(); }
export function main(base) { const options = { nested: { root: new URL("", base) } }; mutate(options.nested.root); return readdirSync(options.nested.root); }
