import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, beforeEach, it } from "vitest";
import { query } from "./query-files.mjs";
it("query", async () => { process.env.X = mkdtempSync(join(tmpdir(), "x-")); await query("table"); });
it("missing", async () => { await query("table"); });
