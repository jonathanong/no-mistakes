import { readdirSync } from "node:fs";
export function list(...args: string[]) { const entries = readdirSync(...args); args[0] = process.cwd(); return entries; }
