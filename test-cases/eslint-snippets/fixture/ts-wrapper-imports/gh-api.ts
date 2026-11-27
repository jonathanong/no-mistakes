import { execFileSync } from "node:child_process";

execFileSync["call" as const](null, "gh", ["api", "repos/o/r?q=1"]);
(require as NodeRequire)("node:child_process").execFileSync("gh", ["api", "repos/o/r?q=1"]);
