import { readdirSync } from "node:fs";
import * as fs from "node:fs";
import { promises } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("passes member promise factories to a filesystem call", () => {
  readdirSync(fs.promises.mkdtemp(join(tmpdir(), "case-")));
  readdirSync(promises.mkdtemp(join(tmpdir(), "member-")));
});
