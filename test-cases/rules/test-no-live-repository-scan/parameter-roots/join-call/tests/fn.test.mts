import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fn } from "../src/fn.mts";

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  fn(join(root, "rules"), join(root, "out"));
} finally {
  rmSync(root, { recursive: true });
}
