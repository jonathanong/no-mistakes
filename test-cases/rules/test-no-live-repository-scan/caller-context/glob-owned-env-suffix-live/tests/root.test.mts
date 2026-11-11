import { vi } from "vitest";
import { main } from "../src/main.mts";
import { count } from "../src/count.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const worktreeRoot = mkdtempSync(join(tmpdir(), "count-"));
vi.stubEnv("SCAN_PROJECT_ROOT", worktreeRoot);
count(worktreeRoot, { root: process.env.SCAN_PROJECT_ROOT });

main(worktreeRoot, { root: process.env.SCAN_PROJECT_ROOT });
