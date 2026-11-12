import { scanArtifact, writeArtifact } from "../src/reassigned-root.mts";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "prefix-"));
  try { run(root); } finally { rmSync(root, { force: true, recursive: true }); }
}

it("rejects unsafe entries", () => {
  withTempDirectory((root) => {
    expect(() => {
      root = process.cwd();
      scanArtifact(root);
    }).toThrow("Unsafe");
  });
});
