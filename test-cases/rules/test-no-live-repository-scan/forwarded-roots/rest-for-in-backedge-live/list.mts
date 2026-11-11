import { readdirSync } from "node:fs";
export function list(...args: string[]) { for (const key in { first: true, second: true }) { readdirSync(...args); args[0] = process.cwd(); } }
