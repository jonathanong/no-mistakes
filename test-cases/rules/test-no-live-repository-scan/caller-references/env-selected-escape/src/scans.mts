import { readdirSync } from "node:fs";
export function scanOwnedEnvironment() {
  return readdirSync(process.env.OWNED_SCAN_ROOT ?? "");
}

export function unrelated() { return "other"; }
