import { vi } from "vitest";
import { readdirSync } from "node:fs";
export function scanOwnedEnvironment() {
  vi.unstubAllEnvs();
  return readdirSync(process.env.OWNED_SCAN_ROOT ?? "");
}
