import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { beforeAll, afterAll, it, describe } from "vitest";
import { query } from "./query-files.mjs";
let release;
const finished = new Promise((resolve) => { release = resolve; });
describe.concurrent("cleans first", () => {
  beforeAll(() => { process.env.X = mkdtempSync(join(tmpdir(), "a-")); });
  afterAll(() => { delete process.env.X; release(); });
  it("finishes", () => {});
});
describe.concurrent("reads after cleanup", () => {
  beforeAll(() => { process.env.X = mkdtempSync(join(tmpdir(), "b-")); });
  it("query", async () => { await finished; await query("table"); });
});
