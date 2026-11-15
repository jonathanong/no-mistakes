import { mkdtempSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function scanFixture(root = process.cwd()) {
  return readdirSync(root);
}

test("owned", () => {
  const root = mkdtempSync(join(tmpdir(), "a-"));
  scanFixture(root);
});

test("omitted", () => {
  scanFixture();
});
