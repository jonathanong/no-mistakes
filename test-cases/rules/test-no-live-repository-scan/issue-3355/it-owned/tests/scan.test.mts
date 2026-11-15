import { mkdtempSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { it } from "vitest";

function scanFixture(root: string) {
  return readdirSync(root);
}

it("a", () => {
  const root = mkdtempSync(join(tmpdir(), "a-"));
  scanFixture(root);
});

it("b", () => {
  const root = mkdtempSync(join(tmpdir(), "b-"));
  scanFixture(root);
});
