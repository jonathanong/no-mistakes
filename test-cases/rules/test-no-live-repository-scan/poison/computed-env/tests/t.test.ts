// Reported: `process["env"]` is a spelling the recorder does not name.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
process["env"].POISON_DIR = "./src";
list();
