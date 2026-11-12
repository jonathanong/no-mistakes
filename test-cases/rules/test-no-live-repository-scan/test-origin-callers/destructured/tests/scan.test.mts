import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { check } from "../src/scan.mts";
const root = mkdtempSync(join(tmpdir(), "owned-"));
check({ root, paths: ["src"] });
