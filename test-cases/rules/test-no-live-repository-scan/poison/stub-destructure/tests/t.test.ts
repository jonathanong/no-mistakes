// Reported: a destructured stubEnv overrides the owned write.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const { stubEnv } = vi;
stubEnv("POISON_DIR", "./src");
list();
