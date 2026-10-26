import { execFile } from "node:child_process";
// This helper is reached across files; its arguments must remain available.
export function execute() { execFile("live", ["--version"]); }
