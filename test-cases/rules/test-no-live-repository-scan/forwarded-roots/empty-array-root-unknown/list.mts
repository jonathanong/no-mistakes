import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
export function list(...args: string[]) { return readdirSync(...args); }
