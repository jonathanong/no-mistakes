// Reported: `const { env } = process` hands out an unnamed alias.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const { env } = process;
env.POISON_DIR = "./src";
list();
