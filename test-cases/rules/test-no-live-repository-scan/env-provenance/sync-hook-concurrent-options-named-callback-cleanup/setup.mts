import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { vi, beforeEach, afterEach } from "vitest";
let release;
const finished = new Promise((resolve) => { release = resolve; });
export function afterReset() { return finished; }
process.env.X = mkdtempSync(join(tmpdir(), "x-"));
beforeEach(() => { vi.stubEnv("X", mkdtempSync(join(tmpdir(), "x-"))); });
afterEach(() => { vi.unstubAllEnvs(); delete process.env.X; release(); });
