import { readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

const fs = require("node:fs");

test("uses a required callback factory result as a path", () => {
  readdirSync(fs.mkdtemp(join(tmpdir(), "case-"), () => {}));
  readdirSync(fs.mkdtemp(join(tmpdir(), "one-")));
});
