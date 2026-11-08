import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { it, vi as v, afterEach } from "vitest";
import { scanOwnedEnvironment } from "../src/scans.mjs";

it("scans an owned environment", () => {
  const root = mkdtempSync(join(tmpdir(), "x-"));
  v.stubEnv("OWNED_SCAN_ROOT", root);
  scanOwnedEnvironment();
});

afterEach(() => v.unstubAllEnvs());
