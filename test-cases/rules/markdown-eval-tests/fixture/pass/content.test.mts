import { readFileSync } from "node:fs";

const body = readFileSync("runbook.md", "utf8");
void body.includes("tofu apply");
