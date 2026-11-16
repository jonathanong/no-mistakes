import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import * as workers from "../src/scan.mts";
const root = mkdtempSync(join(tmpdir(), "scan-"));
const name = "sync";
workers[name](root);
