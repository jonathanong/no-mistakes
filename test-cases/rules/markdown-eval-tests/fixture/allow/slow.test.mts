import { execFileSync } from "node:child_process";

const doc = "runbook.md";
execFileSync("bash", ["-c", "eval \"$block\""]);
void doc;
