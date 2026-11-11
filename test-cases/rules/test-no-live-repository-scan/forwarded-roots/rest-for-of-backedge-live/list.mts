import { readdirSync } from "node:fs";
export function list(...args: string[]) { for (const entry of [0, 1]) { readdirSync(...args); args[0] = process.cwd(); } }
