import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list({ schemaDir = unknownRoot } = {}) { return readdirSync(schemaDir); }
