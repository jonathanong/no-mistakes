import { join } from "node:path";
import { vi } from "vitest";
import { scanFixtureEnv, scanUnconfiguredEnv } from "../src/scan.mts";

const base = process.env.UNKNOWN_BASE;
vi.stubEnv("SCAN_ROOT", join(base ?? ".", "samples"));
scanFixtureEnv();
vi.stubEnv("LIVE_ROOT", join(base ?? ".", "workflows"));
scanUnconfiguredEnv();
