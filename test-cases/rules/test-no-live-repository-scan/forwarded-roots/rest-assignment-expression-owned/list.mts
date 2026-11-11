import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: string[]) { void (args[0] = mkdtempSync("/tmp/rest-")); return readdirSync(...args); }
