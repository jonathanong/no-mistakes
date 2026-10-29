// Reported: the global object reaches the Function constructor.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
globalThis.constructor.constructor('return process')().env.POISON_DIR = './src';
list();
