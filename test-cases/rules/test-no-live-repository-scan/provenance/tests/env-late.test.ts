// Reported: the owned assignment runs after the call, so it does not dominate.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listLate } from "../src/env-late";

listLate();
process.env.LATE_DIR = mkdtempSync(join(tmpdir(), "env-late-"));
