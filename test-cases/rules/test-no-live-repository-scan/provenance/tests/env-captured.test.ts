// Reported: the owned write comes after the module captured the env value.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listCaptured } from "../src/env-captured";

process.env.CAPTURED_DIR = mkdtempSync(join(tmpdir(), "env-captured-"));
listCaptured();
