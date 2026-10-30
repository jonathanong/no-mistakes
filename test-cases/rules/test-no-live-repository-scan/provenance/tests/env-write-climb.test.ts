// Reported: the env write climbs out of mkdtemp with `..`.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listWritten } from "../src/env-write-climb";

process.env.WRITE_CLIMB_DIR = join(
  mkdtempSync(join(tmpdir(), "env-write-climb-")),
  "..",
  "..",
  "..",
);
listWritten();
