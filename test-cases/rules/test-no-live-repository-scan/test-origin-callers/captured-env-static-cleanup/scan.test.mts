import { beforeEach, afterEach, it, vi } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validate } from "./scan.mts";
beforeEach(() => {
  vi.stubEnv("SCAN_ROOT", mkdtempSync(join(tmpdir(), "owned-")));
});
afterEach(() => {
  vi.unstubAllEnvs();
  validate();
});
it("hooks", () => {});
