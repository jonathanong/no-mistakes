import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, afterAll, beforeAll, it } from "vitest";
import { query } from "./query-files.mjs";
beforeAll(() => { vi.stubEnv("TMPDIR", mkdtempSync(join(tmpdir(), "prefix-"))); });
it("query", async () => { await query("table"); });
afterAll(() => { vi.unstubAllEnvs(); });
