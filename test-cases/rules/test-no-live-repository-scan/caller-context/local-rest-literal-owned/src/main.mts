import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
export function main() { const { ignored, ...rest } = { root: mkdtempSync("/tmp/rest-") }; return readdirSync(rest.root); }
