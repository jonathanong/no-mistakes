import { readdirSync } from "node:fs";
export function scan(options) { return readdirSync(options.nested.root); }
export function main(options) { if (options === null || !options) return []; if (options) return scan(options); return []; }
