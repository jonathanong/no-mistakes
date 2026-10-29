// Reported: a module loaded by a computed specifier may be process.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const name = "node:" + "process";
(await vi.importActual<{ env: Record<string, string> }>(name)).env.POISON_DIR = "./src";
list();
