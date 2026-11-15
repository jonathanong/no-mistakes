import { assertOwned } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

await assertOwned(mkdtempSync(join(tmpdir(), "scan-")));
