import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
void (scan as unknown);
import type * as Types from "../src/scan.mts";
type Scan = typeof Types.scan;
import "../src/types.mts";
scan(".");
