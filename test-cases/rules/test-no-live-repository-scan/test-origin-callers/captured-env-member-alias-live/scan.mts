import { readdirSync } from "node:fs";
import { join } from "node:path";
const options = { root: process.env.SCAN_ROOT! };
const alias = options;
export function validate() { return readdirSync(join(alias.root, "rules")); }
