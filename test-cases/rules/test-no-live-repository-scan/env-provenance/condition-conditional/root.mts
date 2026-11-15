import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, it } from "vitest";
import { scan } from "./scan.mjs";

if (vi.stubEnv("OWNED_SCAN_ROOT", mkdtempSync(join(tmpdir(), "owned-")))) {
}

it("scan", () => {
  scan();
});
