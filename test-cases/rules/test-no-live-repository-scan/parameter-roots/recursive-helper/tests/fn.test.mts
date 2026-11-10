import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { snapshotFiles } from "../src/fn.mts";

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  await snapshotFiles(root);
} finally {
  rmSync(root, { recursive: true });
}
