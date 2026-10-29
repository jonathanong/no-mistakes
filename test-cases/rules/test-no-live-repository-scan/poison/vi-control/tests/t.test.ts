// Accepted: `vi` is only used through direct helper calls.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
vi.fn();
vi.spyOn(console, "log");
vi.restoreAllMocks();
list();
