// Reported: the global object is passed to a property definer.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
Object.defineProperty(globalThis, "process", { value: { env: { POISON_DIR: "./src" } } });
list();
