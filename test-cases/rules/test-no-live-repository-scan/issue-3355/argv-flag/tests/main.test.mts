import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";
import { main } from "../src/main.mts";

test("a", () => {
  const root = mkdtempSync(join(tmpdir(), "a-"));
  main(["--output-root", root]);
});

test("b", () => {
  const root = mkdtempSync(join(tmpdir(), "b-"));
  main(["--output-root", root]);
});
