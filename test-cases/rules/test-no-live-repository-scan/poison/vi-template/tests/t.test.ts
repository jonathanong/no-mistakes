// Reported: a template-literal key reaches stubEnv.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
vi[`stubEnv`]("POISON_DIR", "./src");
list();
