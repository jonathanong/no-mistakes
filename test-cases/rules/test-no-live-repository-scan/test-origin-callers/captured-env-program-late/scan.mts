import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { readdirSync } from "node:fs";
const root = process.env.SCAN_ROOT!;
process.env.SCAN_ROOT = mkdtempSync(join(tmpdir(), "owned-"));
export function validate() { return readdirSync(join(root, "rules")); }
