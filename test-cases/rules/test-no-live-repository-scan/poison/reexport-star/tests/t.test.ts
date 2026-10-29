// Reported: a helper re-exports every process member.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";
import { env } from "./envmod";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
env.POISON_DIR = './src';
list();
