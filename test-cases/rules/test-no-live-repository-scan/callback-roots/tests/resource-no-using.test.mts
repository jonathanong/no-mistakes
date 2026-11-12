import { scanRepository } from "../src/resource-no-using.mts";
import { mkdtempDisposableSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  const resource = mkdtempDisposableSync(join(tmpdir(), "callback-"));
  run(resource.path);
}

test("scans the supplied root", () => {
  withTempDirectory((root) => {
    scanRepository(root);
  });
});
