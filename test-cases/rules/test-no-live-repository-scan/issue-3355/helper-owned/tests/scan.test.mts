import { mkdtempSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function scanFixture(root: string) {
  return readdirSync(root);
}

test("a", () => {
  const root = mkdtempSync(join(tmpdir(), "a-"));
  scanFixture(root);
});

test("b", () => {
  const root = mkdtempSync(join(tmpdir(), "b-"));
  scanFixture(root);
});
