// Reported: `globalThis.process` reaches env without the `process` global.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
globalThis.process.env.POISON_DIR = "./src";
list();
