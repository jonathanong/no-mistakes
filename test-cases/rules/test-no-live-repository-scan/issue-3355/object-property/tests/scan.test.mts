import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";
import { main } from "../src/scan.mts";

test("a", () => {
  const root = mkdtempSync(join(tmpdir(), "a-"));
  main({ outputRoot: root });
});

test("b", () => {
  const root = mkdtempSync(join(tmpdir(), "b-"));
  main({ outputRoot: root });
});
