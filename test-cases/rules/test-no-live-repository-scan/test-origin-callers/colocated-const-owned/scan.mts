import { readdirSync } from "node:fs";
import { join } from "node:path";
export function validate(root: string) { return readdirSync(join(root, "rules")); }
