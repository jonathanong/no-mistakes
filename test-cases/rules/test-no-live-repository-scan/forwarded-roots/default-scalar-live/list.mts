import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(root = process.cwd()) { return readdirSync(root); }
