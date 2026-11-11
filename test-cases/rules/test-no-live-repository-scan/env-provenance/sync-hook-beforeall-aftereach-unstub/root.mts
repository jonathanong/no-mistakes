import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, afterEach, beforeAll, it } from "vitest";
import { query } from "./query-files.mjs";
beforeAll(() => { vi.stubEnv("TMPDIR", mkdtempSync(join(tmpdir(), "prefix-"))); });
it("unrelated", () => {});
it("query again", async () => { await query("table"); });
afterEach(() => { vi.unstubAllEnvs(); });
