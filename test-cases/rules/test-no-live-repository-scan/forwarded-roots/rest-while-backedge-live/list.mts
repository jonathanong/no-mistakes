import { readdirSync } from "node:fs";
export function list(...args: string[]) { let i = 0; while (i++ < 2) { readdirSync(...args); args[0] = process.cwd(); } }
