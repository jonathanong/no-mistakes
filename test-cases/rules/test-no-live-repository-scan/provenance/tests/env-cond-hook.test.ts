// Reported: the hook is registered inside a branch, so it may never run.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { beforeEach, it } from "vitest";
import { listCondHook } from "../src/env-cond-hook";

if (process.argv.length > 1) {
  beforeEach(() => {
    process.env.COND_HOOK_DIR = mkdtempSync(join(tmpdir(), "env-cond-hook-"));
  });
}

it("lists", () => {
  listCondHook();
});
