import { processSync } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

processSync(mkdtempSync(join(tmpdir(), "scan-")));
