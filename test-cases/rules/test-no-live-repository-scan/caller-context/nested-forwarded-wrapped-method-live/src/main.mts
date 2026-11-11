import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
export function main(options) { (options.nested.mutate as () => void)(); return scan(options); }
