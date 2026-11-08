import { vi } from "vitest";
import { readdirSync } from "node:fs";
export function scanOwnedEnvironment() {
  return readdirSync(getRoot());
}

function getRoot() {
  vi.unstubAllEnvs();
  return process.env.OWNED_SCAN_ROOT ?? "";
}
