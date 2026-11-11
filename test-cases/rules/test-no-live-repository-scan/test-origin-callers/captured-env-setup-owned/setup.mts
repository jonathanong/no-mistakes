import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
process.env.SCAN_ROOT = mkdtempSync(join(tmpdir(), "owned-"));
