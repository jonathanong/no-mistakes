// Reported: the assignment is conditional, so it may not have run.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listCond } from "../src/env-conditional";

if (process.argv.length > 2) {
  process.env.COND_DIR = mkdtempSync(join(tmpdir(), "env-cond-"));
}
listCond();
