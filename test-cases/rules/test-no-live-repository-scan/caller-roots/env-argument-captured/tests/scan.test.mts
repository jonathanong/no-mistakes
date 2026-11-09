import { join } from "node:path";
import { vi } from "vitest";
import { scanAfter, scanBefore } from "../src/scan.mts";

const beforeStub = process.env.SCAN_ROOT;
vi.stubEnv("SCAN_ROOT", join(import.meta.dirname, "../samples"));
scanBefore(beforeStub!);

const afterStub = process.env.SCAN_ROOT;
scanAfter(afterStub!);
