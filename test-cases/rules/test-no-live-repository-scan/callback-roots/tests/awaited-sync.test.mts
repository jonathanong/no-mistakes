import { scanRepository } from "../src/awaited-sync.mts";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

async function withTempDirectory(run: (root: string) => Promise<void>) {
  const root = await mkdtempSync(join(tmpdir(), "callback-"));
  try {
    await run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}

test("scans the supplied root", async () => {
  await withTempDirectory(async (root) => {
    scanRepository(root);
  });
});
