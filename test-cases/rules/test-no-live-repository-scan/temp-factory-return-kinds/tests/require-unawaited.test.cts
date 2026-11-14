import { readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

const promises = require("fs/promises");
const fs = require("node:fs");

test("passes a required promise factory to a filesystem call", () => {
  readdirSync(promises.mkdtemp(join(tmpdir(), "case-")));
  readdirSync(fs.promises.mkdtemp(join(tmpdir(), "member-")));
});
