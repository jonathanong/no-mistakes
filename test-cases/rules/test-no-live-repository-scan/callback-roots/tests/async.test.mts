import { scanRepository } from "../src/async.mts";
import { readdirSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

async function withTempDirectory(run: (root: string) => Promise<void>) {
  const root = await mkdtemp(join(tmpdir(), "callback-"));
  try {
    await run(root);
  } finally {
    await rm(root, { recursive: true });
  }
}

test("scans the supplied root", async () => {
  await withTempDirectory(async (root) => {
    scanRepository(root);
  });
});
