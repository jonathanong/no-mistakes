// Reported: a destructuring assignment target writes env.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
({ value: process.env.POISON_DIR } = { value: "./src" });
list();
