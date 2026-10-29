// Reported: require("process") reaches env without a named alias.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
declare const require: (id: string) => unknown;
require("process");
list();
