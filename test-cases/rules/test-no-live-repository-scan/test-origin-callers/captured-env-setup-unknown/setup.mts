import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
declare function externalRoot(): string;
process.env.SCAN_ROOT = externalRoot();
