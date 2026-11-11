import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: string[]) { if (path.resolve(args[0]) === "canonical") args[0] = mkdtempSync("/tmp/rest-"); return readdirSync(...args); }
