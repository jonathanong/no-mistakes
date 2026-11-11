import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list({ schemaDir = process.cwd() } = {}) { return readdirSync(schemaDir); }
