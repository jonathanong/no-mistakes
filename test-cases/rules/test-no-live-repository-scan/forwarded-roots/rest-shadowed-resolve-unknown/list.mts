import { readdirSync, mkdtempSync } from "node:fs";
const resolve = (value: string) => value;
export function list(...args: string[]) { if (resolve(args[0])) args[0] = mkdtempSync("/tmp/rest-"); return readdirSync(...args); }
