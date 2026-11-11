import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list({ root } = { root: mkdtempSync("/tmp/default-") }) { return readdirSync(root); }
