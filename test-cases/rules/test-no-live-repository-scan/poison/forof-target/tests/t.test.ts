// Reported: a for-of target writes env.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
for (process.env.POISON_DIR of ["./src"]) {
  break;
}
list();
