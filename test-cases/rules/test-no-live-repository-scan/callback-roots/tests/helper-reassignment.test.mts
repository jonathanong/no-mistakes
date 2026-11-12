import { scanRepository } from "../src/helper-reassignment.mts";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "callback-"));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}

withTempDirectory = (run) => run(process.cwd());

test("scans the supplied root", () => {
  withTempDirectory((root) => {
    scanRepository(root);
  });
});
