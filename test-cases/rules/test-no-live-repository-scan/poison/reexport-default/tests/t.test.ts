// Reported: a helper re-exports the process default.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";
import proc from "./envmod";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
proc.env.POISON_DIR = './src';
list();
