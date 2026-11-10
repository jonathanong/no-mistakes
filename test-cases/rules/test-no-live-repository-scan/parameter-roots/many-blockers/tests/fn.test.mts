import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fn } from "../src/fn.mts";
import "../src/unreadable-a.cjs";
import "../src/unreadable-b.cjs";

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  fn(root);
} finally {
  rmSync(root, { recursive: true });
}
