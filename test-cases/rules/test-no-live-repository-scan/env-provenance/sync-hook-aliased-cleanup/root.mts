import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, afterEach as cleanup, beforeEach as prepare, it } from "vitest";
import { query } from "./query-files.mjs";
prepare(() => { vi.stubEnv("TMPDIR", mkdtempSync(join(tmpdir(), "prefix-"))); });
it("query", async () => { await query("table"); });
cleanup(() => { vi.unstubAllEnvs(); });
