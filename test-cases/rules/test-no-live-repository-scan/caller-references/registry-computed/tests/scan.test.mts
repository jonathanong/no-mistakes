import { processSync, workers } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const root = mkdtempSync(join(tmpdir(), "scan-"));
const name = "sync";
processSync(root);
workers[name](root);
