// Reported: a variable key reaches an env helper through a cast.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const key: string = "stubEnv";
(vi as any)[key]("POISON_DIR", "./src");
list();
