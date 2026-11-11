import { mkdtemp } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, it } from "vitest";
import { scan } from "./scan.mjs";
it("scan", async () => {
  vi.stubEnv("TMPDIR", process.cwd());
  vi.stubEnv("TEMP", process.cwd());
  await scan();
});
