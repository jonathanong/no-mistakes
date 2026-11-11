import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { production } from "../src/scan.mts";
const root = mkdtempSync(join(tmpdir(), "owned-"));
production();
