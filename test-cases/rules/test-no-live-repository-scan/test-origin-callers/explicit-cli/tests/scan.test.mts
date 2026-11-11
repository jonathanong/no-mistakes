import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validate } from "../src/scan.mts";
const root = mkdtempSync(join(tmpdir(), "owned-"));
validate(root);
