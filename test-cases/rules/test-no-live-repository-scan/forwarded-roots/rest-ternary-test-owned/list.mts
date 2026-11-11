import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: string[]) { args[0] ? true : false; return readdirSync(...args); }
