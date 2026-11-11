import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
declare function consume(value: unknown): void;
import("../src/index.mts").then(consume);
scan(mkdtempSync(join(tmpdir(), "scan-")));
