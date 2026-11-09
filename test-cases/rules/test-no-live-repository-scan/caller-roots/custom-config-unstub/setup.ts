import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";

vi.stubEnv("ROOT_DIR", mkdtempSync(join(tmpdir(), "owned-")));
