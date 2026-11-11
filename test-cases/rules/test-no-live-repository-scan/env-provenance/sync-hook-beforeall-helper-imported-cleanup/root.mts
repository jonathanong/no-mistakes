import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, afterAll, beforeAll, it } from "vitest";
import { query } from "./query-files.mjs";
beforeAll(() => { process.env.X = mkdtempSync(join(tmpdir(), "x-")); });
it("query", async () => { await query("table"); });
afterAll(() => { delete process.env.X; });
