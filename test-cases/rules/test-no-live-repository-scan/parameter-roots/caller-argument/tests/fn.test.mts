import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listFiles } from "../src/list.mts";
import { gather } from "../src/x.mts";

declare const live: string;

gather(live);

const root = mkdtempSync(join(tmpdir(), "p-"));
try {
  listFiles(root);
} finally {
  rmSync(root, { recursive: true });
}
