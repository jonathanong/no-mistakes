import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { it, vi, afterEach } from "vitest";
import { scanOwnedEnvironment } from "../src/scans.mjs";

process.env.OWNED_SCAN_ROOT = process.cwd();
const adapter = { stubEnv() {} };
it("scans an owned environment", () => {
  const root = mkdtempSync(join(tmpdir(), "x-"));
  adapter.stubEnv("OWNED_SCAN_ROOT", root);
  scanOwnedEnvironment();
});

afterEach(() => vi.unstubAllEnvs());
