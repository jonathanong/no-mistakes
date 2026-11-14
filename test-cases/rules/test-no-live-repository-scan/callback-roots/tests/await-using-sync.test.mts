import { scanRepository } from "../src/await-using-sync.mts";
import { mkdtempDisposableSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

async function withTempDirectory(run: (root: string) => Promise<void>) {
  await using directory = mkdtempDisposableSync(join(tmpdir(), "case-"));
  await run(directory.path);
}

test("scans the supplied root", async () => {
  await withTempDirectory(async (root) => {
    scanRepository(root);
  });
});
