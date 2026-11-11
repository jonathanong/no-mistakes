import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
function mutate(url) { url.pathname = process.cwd(); }
export function main(options) { mutate(options.nested.root); return scan(options); }
