import { renamed } from "../src/index.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
renamed(mkdtempSync(join(tmpdir(), "scan-")));
