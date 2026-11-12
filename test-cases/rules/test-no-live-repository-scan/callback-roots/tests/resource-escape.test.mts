import { scanRepository } from "../src/resource-escape.mts";
import { mkdtempDisposableSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

function withTempDirectory(run: (root: string) => void) {
  using resource = mkdtempDisposableSync(join(tmpdir(), "callback-"));
  consume(resource);
  run(resource.path);
}

test("scans the supplied root", () => {
  withTempDirectory((root) => {
    scanRepository(root);
  });
});
