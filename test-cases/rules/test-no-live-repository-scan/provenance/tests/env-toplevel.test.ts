// Reported: the enumeration runs at module load, with no call that an owned write could precede.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { names } from "../src/env-toplevel";

process.env.TOP_DIR = mkdtempSync(join(tmpdir(), "env-top-"));
console.log(names);
