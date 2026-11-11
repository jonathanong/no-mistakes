import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
function scan(options, fallback) { return readdirSync(options.root ?? fallback.root); }
export function main(options, fallback) { return scan(options, fallback); }
