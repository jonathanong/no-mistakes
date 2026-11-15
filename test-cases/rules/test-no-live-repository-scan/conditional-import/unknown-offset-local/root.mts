import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { it, vi } from "vitest";
import { loaded } from "./scan.mjs";

if (enabled) {
  vi.stubEnv("OWNED_SCAN_ROOT", mkdtempSync(join(tmpdir(), "owned-")));
}

it("loads", () => {
  void loaded;
});
