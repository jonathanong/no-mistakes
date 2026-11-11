import { beforeEach, it } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validate } from "./scan.mts";
it("first", () => validate());
it("second", () => validate());
it("third", () => validate());
it("fourth", () => validate());
