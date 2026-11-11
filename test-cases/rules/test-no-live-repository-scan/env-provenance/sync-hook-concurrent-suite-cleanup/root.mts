import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, afterEach, beforeEach, it, describe } from "vitest";
import { query } from "./query-files.mjs";
describe.concurrent("overlapping", () => {
let release;
const finished = new Promise((resolve) => { release = resolve; });
beforeEach(() => { vi.stubEnv("X", mkdtempSync(join(tmpdir(), "x-"))); });
afterEach(() => { vi.unstubAllEnvs(); release(); });
it("query after reset", async () => { await finished; await query("table"); });
it("finishes first", () => {});
});
