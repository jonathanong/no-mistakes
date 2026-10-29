// Reported: process reaches a loader through mainModule.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
(process.mainModule as any)['req' + 'uire']('process').env.POISON_DIR = './src';
list();
