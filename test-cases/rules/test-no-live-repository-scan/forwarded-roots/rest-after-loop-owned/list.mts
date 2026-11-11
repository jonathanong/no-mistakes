import { readdirSync } from "node:fs";
export function list(...args: string[]) { for (let i = 0; i < 2; i++) { readdirSync(...args); } args[0] = process.cwd(); }
