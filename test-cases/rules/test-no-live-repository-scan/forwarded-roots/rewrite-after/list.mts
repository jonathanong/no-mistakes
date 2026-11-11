import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: string[]) { const entries = readdirSync(...args); args[0] = mkdtempSync("/tmp/rewrite-"); return entries; }
