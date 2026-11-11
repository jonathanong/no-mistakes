import { readdirSync } from "node:fs";
import { join } from "node:path";
const options = { root: process.env.SCAN_ROOT! };
export function validate() { return readdirSync(join(options.root, "rules")); }
