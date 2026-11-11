import { readdirSync } from "node:fs";
export function list(...args: string[]) { let i = 0; do { readdirSync(...args); args[0] = process.cwd(); } while (++i < 2); }
