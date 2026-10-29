import { vi } from "vitest";

const key = String(Math.random());
vi.stubEnv(key, "x");
