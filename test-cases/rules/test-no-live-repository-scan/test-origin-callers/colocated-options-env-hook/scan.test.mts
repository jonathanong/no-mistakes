import { beforeEach, it } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validate } from "./scan.mts";
const root = mkdtempSync(join(tmpdir(), "owned-"));
const options = { root: process.env.SCAN_ROOT!, paths: ["rules"] };
beforeEach(() => { process.env.SCAN_ROOT = root; });
it("validates options", () => validate(options));
