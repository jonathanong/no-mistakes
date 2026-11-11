import { readdirSync } from "node:fs";
export function scan(root: string) { return readdirSync(root); }
export function production(root = process.cwd()) { return scan(root); }
