import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, it, vi } from "vitest";
import { scan } from "./scan.mjs";

beforeEach(() => {
  vi.stubEnv("EXAMPLE_LOCAL_DIR", mkdtempSync(join(tmpdir(), "example-")));
});

it("clears the environment root before scanning", () => {
  delete process.env.EXAMPLE_LOCAL_DIR;
  scan();
});

afterEach(() => vi.unstubAllEnvs());
