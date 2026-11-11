import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const module = await import("../src/index.mts");
declare function consume(value: unknown): void;
consume(module);
scan(mkdtempSync(join(tmpdir(), "scan-")));
