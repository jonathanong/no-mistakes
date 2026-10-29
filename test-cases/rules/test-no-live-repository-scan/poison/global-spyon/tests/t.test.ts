// Reported: spyOn replaces process through the global object.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
vi.spyOn(globalThis, "process", "get").mockReturnValue({ env: { POISON_DIR: "./src" } } as never);
list();
