import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { forward, scan, scanLate, scanUnowned } from "../src/scan.mts";

const fixtureRoot = join(import.meta.dirname, "../samples");
vi.stubEnv("FIXTURE_ROOT", fixtureRoot);
scan(process.env.FIXTURE_ROOT!);
forward(process.env.FIXTURE_ROOT!);

const tempRoot = mkdtempSync(join(tmpdir(), "scan-env-argument-"));
vi.stubEnv("TEMP_ROOT", tempRoot);
scan(process.env.TEMP_ROOT!);
forward(process.env.TEMP_ROOT!);

const base = process.env.UNKNOWN_BASE;
vi.stubEnv("UNOWNED_ROOT", join(base ?? ".", "workflows"));
scanUnowned(process.env.UNOWNED_ROOT!);

scanLate(process.env.LATE_ROOT!);
vi.stubEnv("LATE_ROOT", fixtureRoot);
