import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list({ schemaDir = mkdtempSync("/tmp/default-") } = {}) { return readdirSync(schemaDir); }
