import { readdirSync } from "node:fs";
import * as fs from "node:fs";
import { promises } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

test("awaits member promise factories", async () => {
  readdirSync(await fs.promises.mkdtemp(join(tmpdir(), "case-")));
  readdirSync(await promises.mkdtemp(join(tmpdir(), "member-")));
});
