import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi, beforeEach, it } from "vitest";
import { scan } from "./scan.mjs";

beforeEach(() => { vi.stubEnv("EXAMPLE_LOCAL_DIR", mkdtempSync(join(tmpdir(), "example-"))); });
it("scan", () => scan());
