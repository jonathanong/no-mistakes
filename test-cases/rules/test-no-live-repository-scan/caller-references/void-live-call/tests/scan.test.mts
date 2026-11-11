import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
void scan(".");
scan(mkdtempSync(join(tmpdir(), "scan-")));
