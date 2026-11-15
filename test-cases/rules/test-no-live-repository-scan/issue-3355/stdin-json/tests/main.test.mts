import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";
import { main } from "../src/main.mts";

test("a", () => {
  const outputRoot = mkdtempSync(join(tmpdir(), "a-"));
  main(JSON.stringify({ outputRoot }));
});

test("b", () => {
  const outputRoot = mkdtempSync(join(tmpdir(), "b-"));
  main(JSON.stringify({ outputRoot }));
});
