import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
class Mutator { constructor(value) { value.root = process.cwd(); } }
export function main(options) { new Mutator(options.nested); return scan(options); }
