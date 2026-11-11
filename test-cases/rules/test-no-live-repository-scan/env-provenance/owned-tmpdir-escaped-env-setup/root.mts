import { mkdtemp } from "node:fs/promises";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { vi, it } from "vitest";
import { scan } from "./scan.mjs";
it("scan", async () => {
  const fixture = dirname(await mkdtemp(join(tmpdir(), "owned-")));
  vi.stubEnv("TMPDIR", fixture);
  vi.stubEnv("TEMP", fixture);
  await scan();
});
