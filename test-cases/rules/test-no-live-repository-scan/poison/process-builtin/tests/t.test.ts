// Reported: process loads a builtin module.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
process.getBuiltinModule('node:process')!.env.POISON_DIR = './src';
list();
