import { mkdtemp } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, it } from "vitest";
import { scan } from "./scan.mjs";
it("scan", async () => {
  process.env.TMPDIR = process.cwd();
  process.env.TEMP = process.cwd();
  await scan();
});
