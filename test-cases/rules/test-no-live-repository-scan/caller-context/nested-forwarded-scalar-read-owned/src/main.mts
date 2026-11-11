import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
function consume(value) { return value; }
export function main(options) { consume(options.nested.root); return scan(options); }
