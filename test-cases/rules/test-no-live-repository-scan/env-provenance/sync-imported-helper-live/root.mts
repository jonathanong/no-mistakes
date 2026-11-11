import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, beforeEach, it } from "vitest";
import { query } from "./query-files.mjs";
beforeEach(() => { process.env.X = process.cwd(); });
it("query", async () => { await query("table"); });
