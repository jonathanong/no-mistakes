// Accepted: a beforeEach hook assigns an owned root before each test body.
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { beforeEach, it } from "vitest";
import { listHook } from "../src/env-hook";

let dir: string;
beforeEach(async () => {
  dir = await mkdtemp(join(tmpdir(), "env-hook-"));
  process.env.HOOK_DIR = dir;
});

it("lists", async () => {
  await listHook();
});
