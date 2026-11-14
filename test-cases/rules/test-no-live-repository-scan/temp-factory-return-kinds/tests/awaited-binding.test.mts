import { readdirSync } from "node:fs";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("awaits a const bound to the promise factory", async () => {
  const pending = mkdtemp(join(tmpdir(), "case-"));
  readdirSync(await pending);
});
