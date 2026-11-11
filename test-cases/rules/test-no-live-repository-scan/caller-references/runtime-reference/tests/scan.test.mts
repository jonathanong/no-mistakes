import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
declare function consume(value: unknown): void;
void consume(scan as unknown);
scan(mkdtempSync(join(tmpdir(), "scan-")));
