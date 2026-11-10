import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { scan } from "../src/scan.mts";

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  scan({ root, opts: { root: "/var/empty/no-such-live-root" } });
} finally {
  rmSync(root, { recursive: true });
}
