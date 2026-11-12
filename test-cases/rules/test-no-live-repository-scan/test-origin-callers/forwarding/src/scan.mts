import { readdirSync } from "node:fs";
import { join } from "node:path";
export function describeFiles(root: string) { return regularFiles(root); }
function regularFiles(root: string) { return readdirSync(join(root, "src")); }
export function production() { return describeFiles(process.cwd()); }
