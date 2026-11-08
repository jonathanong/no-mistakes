import { readdirSync } from "node:fs";
export function scan() { return readdirSync(process.cwd()); }
