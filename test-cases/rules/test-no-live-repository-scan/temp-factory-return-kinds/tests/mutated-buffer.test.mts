import { readdirSync } from "node:fs";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("does not own a buffer promise that is filled", async () => {
  const pending = mkdtemp(join(tmpdir(), "case-"), { encoding: "buffer" });
  pending.then((root) => root.fill(47));
  readdirSync(await pending);
});
