import { scanRepository } from "../src/spread-after-root.mts";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { it } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "prefix-"));
  try { run(root, ...["unused", process.cwd()]); } finally { rmSync(root, { force: true, recursive: true }); }
}
it("scans the supplied root", () => {
  withTempDirectory((root) => {
    scanRepository(root);
  });
});
