import { vi } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
vi.stubEnv("SCAN_ROOT", mkdtempSync(join(tmpdir(), "owned-")));
