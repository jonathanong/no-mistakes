import { main } from "../src/main.mts";
import { count } from "../src/count.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const worktreeRoot = mkdtempSync(join(tmpdir(), "count-"));
count(worktreeRoot, { test: { root: "/var/live" } });

main(worktreeRoot, { test: { root: "/var/live" } });
