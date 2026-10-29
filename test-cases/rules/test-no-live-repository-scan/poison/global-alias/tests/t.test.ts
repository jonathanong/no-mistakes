// Reported: the global object is aliased, then process is replaced through the alias.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const g = globalThis as { process: unknown };
g.process = { env: { POISON_DIR: "./src" } };
list();
