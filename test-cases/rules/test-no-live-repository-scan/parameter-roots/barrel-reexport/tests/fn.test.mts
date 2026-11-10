import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { enumerate } from "../src/barrel.mts";

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  enumerate(root);
} finally {
  rmSync(root, { recursive: true });
}
