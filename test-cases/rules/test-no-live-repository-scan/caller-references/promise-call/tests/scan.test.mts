import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { forward } from "../src/scan.mts";

await forward(mkdtempSync(join(tmpdir(), "scan-")));
