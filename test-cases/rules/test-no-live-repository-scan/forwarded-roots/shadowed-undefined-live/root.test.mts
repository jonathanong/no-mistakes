import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const root = mkdtempSync(join(tmpdir(), "nullish-"));
const undefined = process.cwd();
readdirSync(undefined ?? root);
