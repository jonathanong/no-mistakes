import { mkdtemp, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("uses the callback factory result as a path", () => {
  readdirSync(mkdtemp(join(tmpdir(), "case-"), () => {}));
});
