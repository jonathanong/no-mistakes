import { scanRepository } from "../src/using-async.mts";
import { mkdtempDisposable } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

async function withTempDirectory(run: (root: string) => Promise<void>) {
  using resource = await mkdtempDisposable(join(tmpdir(), "callback-"));
  await run(resource.path);
}

test("scans the supplied root", async () => {
  await withTempDirectory(async (root) => {
    scanRepository(root);
  });
});
