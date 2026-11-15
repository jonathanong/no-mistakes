import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { assertOwned } from "../src/scan.mts";

await assertOwned(mkdtempSync(join(tmpdir(), "scan-")));
