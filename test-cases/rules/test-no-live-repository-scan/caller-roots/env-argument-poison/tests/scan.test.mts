import { join } from "node:path";
import { vi } from "vitest";
import { scan } from "../src/scan.mts";

vi.stubEnv("SCAN_ROOT", join(import.meta.dirname, "../samples"));
process.env.SCAN_ROOT = process.env.UNKNOWN_ROOT;
scan(process.env.SCAN_ROOT!);
