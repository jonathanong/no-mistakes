import { mkdtemp } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, it } from "vitest";
import { scan } from "./scan.mjs";
it("scan", async () => {
  process.env.DATA_DIR = process.cwd();
  await scan();
});
