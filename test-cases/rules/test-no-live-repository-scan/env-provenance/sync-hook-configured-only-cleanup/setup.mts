import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { beforeEach, afterEach, vi } from "vitest";
beforeEach(() => { vi.stubEnv("X", mkdtempSync(join(tmpdir(), "x-"))); });
afterEach(() => { vi.unstubAllEnvs(); });
