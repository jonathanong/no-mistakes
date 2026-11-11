import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, afterEach, beforeAll, it } from "vitest";
import { query } from "./query-files.mjs";
beforeAll(() => { process.env.X = mkdtempSync(join(tmpdir(), "x-")); });
it("unrelated", () => {});
it("query again", async () => { await query("table"); });
afterEach(() => { delete process.env.X; });
