import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const promise = import("../src/index.mts");
void promise;
scan(mkdtempSync(join(tmpdir(), "scan-")));
