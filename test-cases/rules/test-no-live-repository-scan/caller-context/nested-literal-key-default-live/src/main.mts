import { readdirSync } from "node:fs";
function scan({ "nested.root": root = process.cwd() }) { return readdirSync(root); }
export function main(options) { return scan(options); }
