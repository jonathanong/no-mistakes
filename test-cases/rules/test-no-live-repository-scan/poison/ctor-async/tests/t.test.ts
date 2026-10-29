// Reported: an async function value reaches the AsyncFunction constructor.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
(async () => {}).constructor('process.env.POISON_DIR = "./src"')();
list();
