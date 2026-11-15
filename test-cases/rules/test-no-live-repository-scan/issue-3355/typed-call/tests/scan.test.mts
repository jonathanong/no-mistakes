import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";
import { scan } from "../src/scan.mts";

test("a", () => {
  const root = mkdtempSync(join(tmpdir(), "a-"));
  scan(root);
});

test("b", () => {
  const root = mkdtempSync(join(tmpdir(), "b-"));
  scan(root);
});
