import { readdirSync } from "node:fs";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("awaits the promise factory", async () => {
  readdirSync(await mkdtemp(join(tmpdir(), "case-")));
});
