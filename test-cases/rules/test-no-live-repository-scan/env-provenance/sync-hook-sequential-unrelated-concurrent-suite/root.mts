import { mkdtempSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, afterEach, beforeEach, it, describe } from "vitest";
import { query } from "./query-files.mjs";
describe("sequential", () => {
beforeEach(() => { process.env.X = mkdtempSync(join(tmpdir(), "x-")); });
it("query", async () => { await query("table"); });
afterEach(() => { delete process.env.X; });
});
describe.concurrent("unrelated", () => {
  it("temp files", () => { readdirSync(mkdtempSync(join(tmpdir(), "other-"))); });
});
