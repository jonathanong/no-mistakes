import { scanRepository } from "../src/callback-call.mts";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "callback-"));
  try {
    run.call(null, root);
  } finally {
    console.log("no cleanup");
  }
}

test("scans the supplied root", () => {
  withTempDirectory((root) => {
    scanRepository(root);
  });
});
