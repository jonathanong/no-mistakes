import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fn as alias, RESULT_PREFIX } from "../src/fn.mts";

const label = `${RESULT_PREFIX}case`;
void label;
void alias;
const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  alias(root);
} finally {
  rmSync(root, { recursive: true });
}
