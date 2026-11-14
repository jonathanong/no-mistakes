import { readdirSync } from "node:fs";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("awaits a const alias of the promise factory", async () => {
  const pending = mkdtemp(join(tmpdir(), "case-"));
  const alias = pending;
  readdirSync(await alias);
});
