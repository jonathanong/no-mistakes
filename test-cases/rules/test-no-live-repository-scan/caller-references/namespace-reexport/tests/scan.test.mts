import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { helpers } from "../src/namespace.mts";
declare function consume(value: unknown): void;
consume(helpers);
scan(mkdtempSync(join(tmpdir(), "scan-")));
