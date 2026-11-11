import { readdirSync } from "node:fs";
import { join } from "node:path";
const root = process.env.SCAN_ROOT!;
export function validate() { return readdirSync(join(root, "rules")); }
export default { test: { include: ["*.test.mts"], setupFiles: ["./setup.mts"] } };
