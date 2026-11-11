import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
const root = mkdtempSync(join(tmpdir(), "captured-"));
const captured = root ?? process.env.SCAN_ROOT;
vi.stubEnv("SCAN_ROOT", root);
readdirSync(captured);
