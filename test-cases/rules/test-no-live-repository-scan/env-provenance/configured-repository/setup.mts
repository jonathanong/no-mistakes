import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, beforeEach, it } from "vitest";
vi.stubEnv("OWNED_SCAN_ROOT", process.cwd());
