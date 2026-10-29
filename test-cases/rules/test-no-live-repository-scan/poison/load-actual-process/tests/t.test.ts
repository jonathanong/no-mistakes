// Reported: process is loaded through the mock runner.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
(await vi.importActual<{ env: Record<string, string> }>("node:process")).env.POISON_DIR = "./src";
list();
