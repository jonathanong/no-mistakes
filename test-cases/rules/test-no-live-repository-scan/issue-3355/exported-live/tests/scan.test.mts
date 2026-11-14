import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";
import "../src/entry.mts";
import { scanFixture } from "../src/scan.mts";

test("a", () => {
  const root = mkdtempSync(join(tmpdir(), "a-"));
  scanFixture(root);
});
