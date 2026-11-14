import { mkdtempSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("awaits a synchronous factory", async () => {
  readdirSync(await mkdtempSync(join(tmpdir(), "case-")));
});
