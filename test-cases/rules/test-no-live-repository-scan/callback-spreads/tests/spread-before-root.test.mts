import { scanRepository } from "../src/spread-before-root.mts";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { it } from "vitest";

function withTempDirectory(run: (unused: string, root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "prefix-"));
  try { run(...["unused", process.cwd()], root); } finally { rmSync(root, { force: true, recursive: true }); }
}
it("scans the supplied root", () => {
  withTempDirectory((_unused, root) => {
    scanRepository(root);
  });
});
