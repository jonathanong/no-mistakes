import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, it, vi } from "vitest";
import { scan } from "./scan.mjs";

delete process.env.EXAMPLE_LOCAL_DIR;
beforeEach(() => { vi.stubEnv("EXAMPLE_LOCAL_DIR", mkdtempSync(join(tmpdir(), "example-"))); });
it("scans after resetting the environment to an owned root", () => scan());
afterEach(() => { vi.unstubAllEnvs(); });
