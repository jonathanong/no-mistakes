import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, it } from "vitest";
import { query } from "./query-files.mjs";
function beforeEach(callback) { callback(); }
function afterEach(callback) { callback(); }
beforeEach(() => { vi.stubEnv("TMPDIR", mkdtempSync(join(tmpdir(), "prefix-"))); });
it("query", async () => { await query("table"); });
afterEach(() => { vi.unstubAllEnvs(); });
