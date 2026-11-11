import { beforeEach, it } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validate } from "./scan.mts";
const root = mkdtempSync(join(tmpdir(), "owned-"));
it("validates options", () => {
  const options = { root, paths: ["rules"] };
  Object.assign(options, { root: process.cwd() });
  validate(options);
});
