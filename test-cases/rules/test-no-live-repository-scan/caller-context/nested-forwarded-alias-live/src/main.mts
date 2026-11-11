import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
export function main(options) { const alias = options.nested; alias.root = process.cwd(); return scan(options); }
