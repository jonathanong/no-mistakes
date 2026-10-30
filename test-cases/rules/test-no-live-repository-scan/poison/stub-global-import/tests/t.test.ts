// Reported: stubGlobal("process") replaces process when the binding is not named vi.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { stubGlobal } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
stubGlobal("process", { env: {} });
list();
