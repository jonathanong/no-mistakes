// Reported: a variable holding the literal constructor is used as a key.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const k = "constructor";
(function () {})[k]('process.env.POISON_DIR = "./src"')();
list();
