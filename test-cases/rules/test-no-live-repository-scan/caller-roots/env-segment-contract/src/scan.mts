import { readdirSync } from "node:fs";
import { join } from "node:path";

export function scanFixtureEnv() {
  return readdirSync(join(process.env.SCAN_ROOT ?? "", "items"));
}

export function scanUnconfiguredEnv() {
  return readdirSync(join(process.env.LIVE_ROOT ?? "", "items"));
}
