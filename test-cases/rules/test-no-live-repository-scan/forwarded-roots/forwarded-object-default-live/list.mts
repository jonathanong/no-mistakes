import { readdirSync, mkdtempSync } from "node:fs";
export function list(options = { root: process.cwd() }) { return scan(options); }
function scan({ root = mkdtempSync("/tmp/default-") } = {}) { return readdirSync(root); }
