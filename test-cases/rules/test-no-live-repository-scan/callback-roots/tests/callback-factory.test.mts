import { scanRepository } from "../src/callback-factory.mts";
import { mkdtemp, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const root = mkdtemp(join(tmpdir(), "callback-"), () => {});
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}

test("scans the supplied root", () => {
  withTempDirectory((root) => {
    scanRepository(root);
  });
});
