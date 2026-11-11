import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
export function main(options, key = "root") { options.nested[key] = process.cwd(); return scan(options); }
