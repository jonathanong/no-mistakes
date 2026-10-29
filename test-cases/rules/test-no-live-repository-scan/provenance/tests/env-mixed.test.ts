// Reported: another file writes the same variable with a non-owned value.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listMixed } from "../src/env-mixed";

process.env.MIXED_DIR = mkdtempSync(join(tmpdir(), "env-mixed-"));
listMixed();
