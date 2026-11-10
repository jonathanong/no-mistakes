import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fn } from "../src/fn.mts";
import "../src/unreadable.cjs";

const require = createRequire(import.meta.url);
const configPath = require.resolve("../.dependency-cruiser.cjs");
readFileSync(configPath, "utf8");
execFileSync(process.execPath, ["-e", "process.exit(0)"]);
const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  fn(root);
} finally {
  rmSync(root, { recursive: true });
}
