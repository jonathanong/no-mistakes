import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
process.env.SCAN_ROOT = mkdtempSync(join(tmpdir(), "owned-"));
import { readdirSync } from "node:fs";
const root = process.env.SCAN_ROOT!;
export function validate() { return readdirSync(join(root, "rules")); }
