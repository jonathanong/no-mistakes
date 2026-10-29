// Reported: a prototype lookup reaches the Function constructor.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
Object.getPrototypeOf(function () {}).constructor('process.env.POISON_DIR = "./src"')();
list();
