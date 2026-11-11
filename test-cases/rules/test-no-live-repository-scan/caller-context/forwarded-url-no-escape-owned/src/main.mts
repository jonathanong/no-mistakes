import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
export function scan(options) { return readdirSync(options.nested.root); }
export function main(options) { return scan(options); }
