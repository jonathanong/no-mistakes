import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
export function main(fallback) { function scan(options) { return readdirSync(options.root ?? fallback); } return scan({}); }
