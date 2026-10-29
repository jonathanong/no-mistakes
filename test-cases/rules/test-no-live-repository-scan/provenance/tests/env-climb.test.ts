// Reported: the env var is owned, but the path leaves it through `..`.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listAbove } from "../src/env-climb";

process.env.CLIMB_DIR = mkdtempSync(join(tmpdir(), "env-climb-"));
listAbove();
