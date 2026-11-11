import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
export function main(options) { options.nested.root = process.cwd(); const forwarded = options; return scan(forwarded); }
