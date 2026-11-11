import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: string[]) { args[0] = mkdtempSync("/tmp/rewrite-"); return readdirSync(...args); }
