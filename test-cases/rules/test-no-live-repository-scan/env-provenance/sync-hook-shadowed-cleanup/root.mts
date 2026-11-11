import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, it } from "vitest";
import { query } from "./query-files.mjs";
function register(beforeEach, afterEach) {
beforeEach(() => { vi.stubEnv("TMPDIR", mkdtempSync(join(tmpdir(), "prefix-"))); });
it("query", async () => { await query("table"); });
afterEach(() => { vi.unstubAllEnvs(); });

}
register((callback) => callback(), (callback) => callback());
