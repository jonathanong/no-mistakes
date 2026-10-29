// Reported: `unstubEnvs: true` restores the stub after each test.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { beforeAll, it, vi } from "vitest";
import { list } from "../src/dir";

beforeAll(() => {
  vi.stubEnv("POISON_DIR", mkdtempSync(join(tmpdir(), "poison-")));
});
it("first", () => {
  list();
});
it("second", () => {
  list();
});
