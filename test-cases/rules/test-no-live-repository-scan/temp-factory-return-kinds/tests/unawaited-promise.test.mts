import { readdirSync } from "node:fs";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("passes the promise to a filesystem call", () => {
  readdirSync(mkdtemp(join(tmpdir(), "case-")));
});
