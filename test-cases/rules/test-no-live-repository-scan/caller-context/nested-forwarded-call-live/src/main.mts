import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
function mutate(value) { value.root = process.cwd(); }
export function main(options) { mutate(options.nested); return scan(options); }
