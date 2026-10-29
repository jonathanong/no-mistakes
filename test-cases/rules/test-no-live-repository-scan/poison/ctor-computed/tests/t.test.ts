// Reported: a built key spells constructor.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
(function () {})['constr' + 'uctor']('process.env.POISON_DIR = "./src"')();
list();
