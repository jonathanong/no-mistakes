import { main } from "../src/main.mts";
import { count } from "../src/count.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const worktreeRoot = mkdtempSync(join(tmpdir(), "count-"));
count(worktreeRoot, { root: "../repo" });

main(worktreeRoot, { root: "../repo" });
