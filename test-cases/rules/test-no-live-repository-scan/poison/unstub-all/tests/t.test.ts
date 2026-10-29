// Reported: unstubAllEnvs restores the default between tests.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeAll, it, vi } from "vitest";
import { list } from "../src/dir";

beforeAll(() => {
  vi.stubEnv("POISON_DIR", mkdtempSync(join(tmpdir(), "poison-")));
});
afterEach(() => {
  vi.unstubAllEnvs();
});
it("first", () => {
  list();
});
it("second", () => {
  list();
});
