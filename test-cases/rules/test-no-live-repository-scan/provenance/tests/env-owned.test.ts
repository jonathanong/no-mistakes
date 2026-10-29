// Accepted: the test owns the directory the env var names, and the
// assignment precedes the call that reaches the getter and the join.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listTable } from "../src/env-owned";

const testDir = mkdtempSync(join(tmpdir(), "env-owned-"));
process.env.OWNED_DIR = testDir;
listTable("events");
