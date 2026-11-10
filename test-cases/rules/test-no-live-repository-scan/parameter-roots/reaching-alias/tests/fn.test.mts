import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fn, RESULT_PREFIX } from "../src/fn.mts";

const label = `${RESULT_PREFIX}case`;
void label;
const alias = fn;
void alias;
const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  fn(root);
} finally {
  rmSync(root, { recursive: true });
}
