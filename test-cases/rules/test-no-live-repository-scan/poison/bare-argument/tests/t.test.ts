// Reported: process.env passed to a function may be mutated there.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
function clobber(target: Record<string, string | undefined>) {
  target.POISON_DIR = "./src";
}
clobber(process.env);
list();
