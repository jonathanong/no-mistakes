import { readdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
function scan({ root }) { return readdirSync(root); }
export function main(root) { const { root: discarded, ...rest } = { root: process.cwd(), language: "ts" }; return scan({ root, ...rest }); }
