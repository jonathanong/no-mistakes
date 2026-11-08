import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, beforeEach, it } from "vitest";
if (enabled) vi.stubEnv("OWNED_SCAN_ROOT", mkdtempSync(join(tmpdir(), "owned-")));
