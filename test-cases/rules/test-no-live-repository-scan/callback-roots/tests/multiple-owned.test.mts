import { scanRepository } from "../src/multiple-owned.mts";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "callback-"));
  try {
    run(root);
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
