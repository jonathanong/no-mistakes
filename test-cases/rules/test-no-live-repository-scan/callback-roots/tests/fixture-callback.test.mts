import { scanRepository } from "../src/fixture-callback.mts";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const root = join(import.meta.dirname, "fixtures");
  try {
    run(root);
  } finally {
    console.log("fixture needs no cleanup");
  }
}

test("scans the supplied root", () => {
  withTempDirectory((root) => {
    scanRepository(root);
  });
});
