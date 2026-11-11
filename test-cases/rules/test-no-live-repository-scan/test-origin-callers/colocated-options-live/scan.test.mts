import { beforeEach, it } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validate } from "./scan.mts";
const root = process.cwd();
it("validates options", () => {
  const options = { root, paths: ["rules"] };
  validate(options);
});
