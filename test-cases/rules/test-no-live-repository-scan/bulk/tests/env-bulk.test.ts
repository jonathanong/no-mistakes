// Reported: a bulk write to process.env cannot be attributed to a variable.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listBulk } from "../src/env-bulk";

process.env.BULK_DIR = mkdtempSync(join(tmpdir(), "env-bulk-"));
Object.assign(process.env, { OTHER: "x" });
listBulk();
