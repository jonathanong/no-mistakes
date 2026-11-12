import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describeFiles } from "../src/scan.mts";
import { suppliedRoot } from "external-input";
describeFiles(suppliedRoot);
describeFiles(mkdtempSync(join(tmpdir(), "owned-")));
