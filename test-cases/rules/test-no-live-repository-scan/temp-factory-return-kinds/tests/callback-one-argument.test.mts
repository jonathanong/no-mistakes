import { mkdtemp, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("uses a one-argument callback factory as a path", () => {
  readdirSync(mkdtemp(join(tmpdir(), "case-")));
});
