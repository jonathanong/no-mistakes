import { readdirSync, mkdtempSync } from "node:fs";
export function list(seed: { root: string }, options = seed) { return scan(options); }
function scan({ root = mkdtempSync("/tmp/default-") } = {}) { return readdirSync(root); }
