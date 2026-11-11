import "../src/entry.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { runSmoke } from "../src/scan.mts";

const root = mkdtempSync(join(tmpdir(), "smoke-"));
runSmoke(root);
