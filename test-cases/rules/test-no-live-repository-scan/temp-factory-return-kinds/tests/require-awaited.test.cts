import { readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

const promises = require("fs/promises");
const fs = require("node:fs");

test("awaits required promise factories and a synchronous string", async () => {
  readdirSync(await promises.mkdtemp(join(tmpdir(), "case-")));
  readdirSync(await fs.promises.mkdtemp(join(tmpdir(), "member-")));
  readdirSync(await fs.mkdtempSync(join(tmpdir(), "sync-")));
});
