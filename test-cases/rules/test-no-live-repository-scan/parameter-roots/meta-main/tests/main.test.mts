import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { main } from "../src/main.mts";

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  main(root);
} finally {
  rmSync(root, { recursive: true });
}
