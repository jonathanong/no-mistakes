// Reported: `.call` shifts the argument positions, so `dir` is "./src" here.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { syncCalled } from "../src/param-call";

const ownedDir = mkdtempSync(join(tmpdir(), "param-call-"));
syncCalled("admin", ownedDir);
syncCalled.call(null, ownedDir, "./src");
