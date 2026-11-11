import { readdirSync, mkdtempSync } from "node:fs";
export function list({ root = mkdtempSync("/tmp/default-") } = { root: process.cwd() }) { return readdirSync(root); }
