import { scanRepository } from "../src/escape.mts";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

const roots: string[] = [];
function withTempDirectory(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "callback-"));
  try {
    roots.push(root);
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
