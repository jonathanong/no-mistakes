import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { it, vi, afterEach } from "vitest";
import { scanOwnedEnvironment } from "../src/scans.mjs";

it("scans an owned environment", async () => {
  const root = mkdtempSync(join(tmpdir(), "x-"));
  vi.stubEnv("OWNED_SCAN_ROOT", root);
  await Promise.resolve();
  scanOwnedEnvironment();
});

afterEach(() => vi.unstubAllEnvs());
