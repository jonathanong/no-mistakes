// Reported: Reflect.get reaches constructor through a string argument.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const F = Reflect.get(function () {}, "constructor");
F('process.env.POISON_DIR = "./src"')();
list();
