import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
const root = mkdtempSync(join(tmpdir(), "climb-"));
const otherRoot = mkdtempSync(join(tmpdir(), "dirname-other-"));
readdirSync(dirname(root ?? otherRoot));
