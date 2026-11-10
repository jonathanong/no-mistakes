import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/list.mts";

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  list({ outputRoot: root });
} finally {
  rmSync(root, { recursive: true });
}

list({ outputRoot: "/var/empty/no-such-live-root" });
