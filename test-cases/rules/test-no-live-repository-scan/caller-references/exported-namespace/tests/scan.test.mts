import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
export const module = await import("../src/index.mts");
scan(mkdtempSync(join(tmpdir(), "scan-")));
