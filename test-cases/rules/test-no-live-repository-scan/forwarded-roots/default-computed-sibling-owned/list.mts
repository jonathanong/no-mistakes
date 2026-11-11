import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list({ ["other"]: other, root = mkdtempSync("/tmp/default-") } = {}) { return readdirSync(root); }
