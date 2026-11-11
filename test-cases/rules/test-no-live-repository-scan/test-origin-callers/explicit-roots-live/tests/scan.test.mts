import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describeFiles } from "../src/scan.mts";
describeFiles(process.cwd());
describeFiles(mkdtempSync(join(tmpdir(), "owned-")));
