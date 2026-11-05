import { readdirSync } from "node:fs";
export function scan(dir: string) { return readdirSync(dir); }
